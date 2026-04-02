import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// Low-level FTP client over a raw TCP socket.
/// Handles both single-line (e.g. "200 OK") and multi-line responses
/// (e.g. "211-Features:\r\n ...\r\n211 End\r\n").
class FtpClient {
  final String host;
  final int port;

  Socket? _controlSocket;
  StreamSubscription? _controlSubscription;

  // Raw byte accumulator — we decode line by line
  final List<int> _byteBuffer = [];

  // Queue of pending completers, one per command
  final List<Completer<String>> _responseQueue = [];

  // Multi-line response accumulator
  String? _multilineCode;       // e.g. "211"
  final StringBuffer _multilineBuffer = StringBuffer();

  FtpClient(this.host, {this.port = 8080});

  // ── Connection ──────────────────────────────────────────────────────────────

  Future<void> connect() async {
    _controlSocket = await Socket.connect(host, port);
    _controlSocket!.setOption(SocketOption.tcpNoDelay, true);

    _controlSubscription = _controlSocket!.listen(
      _onData,
      onError: _onError,
      onDone: _onDone,
      cancelOnError: false,
    );

    final welcome = await _waitForResponse();
    if (!welcome.startsWith('220')) {
      throw Exception('Unexpected FTP banner: $welcome');
    }
  }

  void _onData(List<int> data) {
    _byteBuffer.addAll(data);
    _processBuffer();
  }

  void _onError(Object e) {
    for (final c in _responseQueue) {
      if (!c.isCompleted) c.completeError(e);
    }
    _responseQueue.clear();
  }

  void _onDone() {
    final err = Exception('Control socket closed unexpectedly');
    for (final c in _responseQueue) {
      if (!c.isCompleted) c.completeError(err);
    }
    _responseQueue.clear();
  }

  /// Processes the raw byte buffer into complete FTP response lines.
  /// Handles both:
  ///   - Single-line:  "200 OK\r\n"
  ///   - Multi-line:   "211-Features:\r\n ...\r\n211 End\r\n"
  void _processBuffer() {
    while (true) {
      // Find next \n
      final nl = _byteBuffer.indexOf(0x0A); // '\n'
      if (nl == -1) break;

      // Extract line (strip \r\n)
      final lineBytes = _byteBuffer.sublist(0, nl);
      _byteBuffer.removeRange(0, nl + 1);

      // Remove trailing \r if present
      final trimmed = (lineBytes.isNotEmpty && lineBytes.last == 0x0D)
          ? lineBytes.sublist(0, lineBytes.length - 1)
          : lineBytes;

      final line = utf8.decode(trimmed, allowMalformed: true);

      _handleLine(line);
    }
  }

  void _handleLine(String line) {
    // Multi-line continuation: "XYZ-..."  (dash after code)
    if (line.length >= 4 && RegExp(r'^\d{3}-').hasMatch(line)) {
      final code = line.substring(0, 3);
      if (_multilineCode == null) {
        // Starting a multi-line response
        _multilineCode = code;
        _multilineBuffer.clear();
        _multilineBuffer.writeln(line);
      } else {
        _multilineBuffer.writeln(line);
      }
      return;
    }

    // Final line of a multi-line response: "XYZ <text>" (space after code)
    if (_multilineCode != null && line.length >= 3 && line.startsWith(_multilineCode!)) {
      _multilineBuffer.writeln(line);
      final full = _multilineBuffer.toString().trim();
      _multilineCode = null;
      _multilineBuffer.clear();
      _deliver(full);
      return;
    }

    // Plain single-line response: "XYZ ..."
    if (line.length >= 4 && RegExp(r'^\d{3} ').hasMatch(line)) {
      if (_multilineCode != null) {
        // Orphan line inside multi-line — accumulate
        _multilineBuffer.writeln(line);
      } else {
        _deliver(line);
      }
      return;
    }

    // Intermediate line inside a multi-line block (no code prefix)
    if (_multilineCode != null) {
      _multilineBuffer.writeln(line);
    }
    // else: ignore server info lines we don't handle
  }

  void _deliver(String response) {
    if (_responseQueue.isNotEmpty) {
      final c = _responseQueue.removeAt(0);
      if (!c.isCompleted) c.complete(response);
    }
  }

  Future<String> _waitForResponse() {
    final completer = Completer<String>();
    _responseQueue.add(completer);
    return completer.future;
  }

  // ── Command layer ───────────────────────────────────────────────────────────

  Future<String> sendCommand(String command) async {
    if (_controlSocket == null) throw Exception('Not connected');
    _controlSocket!.write('$command\r\n');
    return await _waitForResponse();
  }

  // ── High-level helpers ──────────────────────────────────────────────────────

  Future<void> login(String user, String pass) async {
    String res = await sendCommand('USER $user');
    if (!res.startsWith('331')) {
      throw Exception('USER failed: $res');
    }
    res = await sendCommand('PASS $pass');
    if (!res.startsWith('230')) {
      throw Exception('Login failed: $res');
    }
  }

  /// Opens a passive data connection and returns the connected data socket.
  Future<Socket> enterPassiveMode() async {
    final res = await sendCommand('PASV');
    if (!res.startsWith('227')) {
      throw Exception('PASV failed: $res');
    }
    final match = RegExp(r'\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)').firstMatch(res);
    if (match == null) throw Exception('Cannot parse PASV response: $res');

    final ip =
        '${match.group(1)}.${match.group(2)}.${match.group(3)}.${match.group(4)}';
    final p1 = int.parse(match.group(5)!);
    final p2 = int.parse(match.group(6)!);
    final dataPort = (p1 * 256) + p2;

    return await Socket.connect(ip, dataPort);
  }

  // ── Transfers ───────────────────────────────────────────────────────────────

  /// Correct FTP upload sequence: PASV → STOR → open data → send → close → wait 226.
  Future<void> uploadFile(
    String remoteName,
    File localFile, {
    void Function(int sent, int total)? onProgress,
  }) async {
    // 1. Enter passive mode FIRST
    final dataSocket = await enterPassiveMode();

    // 2. Send STOR command
    final storRes = await sendCommand('STOR $remoteName');
    if (!storRes.startsWith('150') && !storRes.startsWith('125')) {
      await dataSocket.close();
      throw Exception('STOR rejected: $storRes');
    }

    // 3. Stream file data
    final total = await localFile.length();
    int sent = 0;
    await for (final chunk in localFile.openRead()) {
      dataSocket.add(chunk);
      sent += chunk.length;
      onProgress?.call(sent, total);
    }
    await dataSocket.flush();
    await dataSocket.close();

    // 4. Wait for 226 Transfer complete
    final doneRes = await _waitForResponse();
    if (!doneRes.startsWith('226')) {
      throw Exception('STOR did not complete: $doneRes');
    }
  }

  /// Download file: PASV → RETR → read data → wait 226.
  Future<void> downloadFile(
    String remoteName,
    String localPath, {
    void Function(int received, int total)? onProgress,
  }) async {
    // 1. Passive mode
    final dataSocket = await enterPassiveMode();

    // 2. Send RETR
    final retrRes = await sendCommand('RETR $remoteName');
    if (!retrRes.startsWith('150') && !retrRes.startsWith('125')) {
      await dataSocket.close();
      throw Exception('RETR rejected: $retrRes');
    }

    // Parse file size from "150 Opening ... (1234 bytes)."
    int total = 0;
    final sizeMatch = RegExp(r'\((\d+) bytes\)').firstMatch(retrRes);
    if (sizeMatch != null) {
      total = int.parse(sizeMatch.group(1)!);
    }

    // 3. Receive data
    final file = File(localPath);
    final sink = file.openWrite();
    int received = 0;
    await for (final chunk in dataSocket) {
      sink.add(chunk);
      received += chunk.length;
      if (total > 0) onProgress?.call(received, total);
    }
    await sink.flush();
    await sink.close();
    await dataSocket.close();

    // 4. Wait for 226
    final doneRes = await _waitForResponse();
    if (!doneRes.startsWith('226')) {
      throw Exception('RETR did not complete: $doneRes');
    }
  }

  // ── Directory listing ───────────────────────────────────────────────────────

  /// Returns raw LIST lines.
  Future<List<String>> listDirectory([String? path]) async {
    final dataSocket = await enterPassiveMode();
    final cmd = path != null ? 'LIST $path' : 'LIST';
    final res = await sendCommand(cmd);
    if (!res.startsWith('150') && !res.startsWith('125')) {
      await dataSocket.close();
      throw Exception('LIST rejected: $res');
    }

    final buffer = StringBuffer();
    await for (final chunk in dataSocket) {
      buffer.write(utf8.decode(chunk, allowMalformed: true));
    }
    await dataSocket.close();

    final doneRes = await _waitForResponse();
    if (!doneRes.startsWith('226')) {
      throw Exception('LIST did not complete: $doneRes');
    }

    return buffer
        .toString()
        .split('\n')
        .map((e) => e.trim())
        .where((e) => e.isNotEmpty)
        .toList();
  }

  // ── Navigation ──────────────────────────────────────────────────────────────

  Future<String> pwd() async {
    final res = await sendCommand('PWD');
    // 257 "/current/dir"
    final match = RegExp(r'"([^"]*)"').firstMatch(res);
    return match?.group(1) ?? '/';
  }

  Future<void> cwd(String path) async {
    final res = await sendCommand('CWD $path');
    if (!res.startsWith('250')) throw Exception('CWD failed: $res');
  }

  Future<void> cdup() async {
    final res = await sendCommand('CDUP');
    if (!res.startsWith('250')) throw Exception('CDUP failed: $res');
  }

  // ── File management ─────────────────────────────────────────────────────────

  Future<void> mkdir(String name) async {
    final res = await sendCommand('MKD $name');
    if (!res.startsWith('257')) throw Exception('MKD failed: $res');
  }

  Future<void> rmdir(String name) async {
    final res = await sendCommand('RMD $name');
    if (!res.startsWith('250')) throw Exception('RMD failed: $res');
  }

  Future<void> deleteFile(String name) async {
    final res = await sendCommand('DELE $name');
    if (!res.startsWith('250')) throw Exception('DELE failed: $res');
  }

  Future<void> rename(String from, String to) async {
    final r1 = await sendCommand('RNFR $from');
    if (!r1.startsWith('350')) throw Exception('RNFR failed: $r1');
    final r2 = await sendCommand('RNTO $to');
    if (!r2.startsWith('250')) throw Exception('RNTO failed: $r2');
  }

  Future<int?> size(String filename) async {
    final res = await sendCommand('SIZE $filename');
    if (res.startsWith('213')) {
      return int.tryParse(res.substring(4).trim());
    }
    return null;
  }

  Future<void> noop() async {
    await sendCommand('NOOP');
  }

  // ── Disconnect ──────────────────────────────────────────────────────────────

  Future<void> disconnect() async {
    if (_controlSocket != null) {
      try {
        _controlSocket!.write('QUIT\r\n');
        await Future.delayed(const Duration(milliseconds: 200));
      } catch (_) {}
      await _controlSubscription?.cancel();
      await _controlSocket!.close();
      _controlSocket = null;
    }
  }
}

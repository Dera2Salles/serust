/// Represents a single entry in an FTP directory listing.
class FtpEntry {
  final String name;
  final bool isDirectory;
  final int size;
  final String rawLine;

  const FtpEntry({
    required this.name,
    required this.isDirectory,
    this.size = 0,
    this.rawLine = '',
  });

  /// Parses a Unix-style LIST response line:
  ///   drwxr-xr-x 1 ftp ftp       4096 Jan 01 00:00 dirname
  ///   -rw-r--r-- 1 ftp ftp      12345 Jan 01 00:00 filename.txt
  factory FtpEntry.fromListLine(String line) {
    if (line.trim().isEmpty) {
      throw FormatException('Empty line');
    }

    final isDir = line.startsWith('d');

    // Split on whitespace, max 9 parts (name may contain spaces)
    final parts = line.trim().split(RegExp(r'\s+'));

    // We need at least 9 parts: perms links user group size month day time name
    String name;
    int size = 0;

    if (parts.isNotEmpty && parts.length >= 9) {
      name = parts.sublist(8).join(' ');
      size = int.tryParse(parts[4]) ?? 0;
    } else if (parts.isNotEmpty) {
      // Fallback: just use the last token
      name = parts.last;
    } else {
      throw FormatException('Cannot parse LIST line: $line');
    }

    // Skip hidden entries and navigation entries
    if (name == '.' || name == '..') {
      throw FormatException('Navigation entry: $name');
    }

    return FtpEntry(name: name, isDirectory: isDir, size: size, rawLine: line);
  }

  @override
  String toString() =>
      '${isDirectory ? '[DIR]' : '[FILE]'} $name ($size bytes)';
}

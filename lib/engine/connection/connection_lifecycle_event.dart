enum ConnectionLifecycleStatus {
  initiated,
  connected,
  disconnected,
  rejected,
  error,
}

class ConnectionLifecycleEvent {
  final ConnectionLifecycleStatus status;
  final String endpointId;
  final String? endpointName;

  ConnectionLifecycleEvent({
    required this.status,
    required this.endpointId,
    this.endpointName,
  });

  @override
  String toString() =>
      'ConnectionLifecycleEvent(status: $status, id: $endpointId, name: $endpointName)';
}

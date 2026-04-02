import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:arosaina/engine/connection/connection_lifecycle_event.dart';
import 'package:arosaina/engine/connection/i_server_connection_repository.dart';

// ── Events ─────────────────────────────────────────────────────────────────
abstract class ConnectionEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class ConnectToServer extends ConnectionEvent {}

class Disconnect extends ConnectionEvent {}

class ConnectionLifecycleChanged extends ConnectionEvent {
  final ConnectionLifecycleEvent event;
  ConnectionLifecycleChanged(this.event);
  @override
  List<Object> get props => [event];
}

// ── States ─────────────────────────────────────────────────────────────────
abstract class ConnectionState extends Equatable {
  @override
  List<Object> get props => [];
}

class ConnectionInitial extends ConnectionState {}

class ConnectionLoading extends ConnectionState {}

class ConnectionConnected extends ConnectionState {}

class ConnectionFailure extends ConnectionState {
  final String error;
  ConnectionFailure(this.error);
  @override
  List<Object> get props => [error];
}

// ── Bloc ───────────────────────────────────────────────────────────────────
class ConnectionBloc extends Bloc<ConnectionEvent, ConnectionState> {
  final IServerConnectionRepository _repository;

  bool get hasActiveConnection => state is ConnectionConnected;

  ConnectionBloc(this._repository) : super(ConnectionInitial()) {
    on<ConnectToServer>((event, emit) async {
      emit(ConnectionLoading());
      try {
        await _repository.connect();
      } catch (e) {
        emit(ConnectionFailure(e.toString()));
      }
    });

    on<ConnectionLifecycleChanged>((event, emit) {
      switch (event.event.status) {
        case ConnectionLifecycleStatus.connected:
          emit(ConnectionConnected());
          break;
        case ConnectionLifecycleStatus.disconnected:
          emit(ConnectionInitial());
          break;
        case ConnectionLifecycleStatus.rejected:
        case ConnectionLifecycleStatus.error:
          emit(ConnectionFailure('Connection error'));
          break;
        default:
          break;
      }
    });

    on<Disconnect>((event, emit) async {
      await _repository.disconnect();
      emit(ConnectionInitial());
    });

    _repository.connectionLifecycle.listen((event) {
      add(ConnectionLifecycleChanged(event));
    });
  }
}

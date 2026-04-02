import 'dart:io';
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../../../engine/transfert/received_transfer.dart';
import '../../../engine/transfert/transfer_tree_node.dart';
import '../../../engine/transfert/i.transfer.repository.dart';

// Events
abstract class TransferEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class SendFile extends TransferEvent {
  final File file;
  SendFile(this.file);
}

class FetchReceivedTransfers extends TransferEvent {}

class DownloadReceivedFile extends TransferEvent {
  final ReceivedTransfer transfer;
  DownloadReceivedFile(this.transfer);
}

class FetchTransferTree extends TransferEvent {
  final String fileId;
  FetchTransferTree(this.fileId);
}

// States
abstract class TransferState extends Equatable {
  @override
  List<Object> get props => [];
}

class TransferInitial extends TransferState {}

class TransferInProgress extends TransferState {
  final double progress;
  TransferInProgress(this.progress);
}

class TransfersLoaded extends TransferState {
  final List<ReceivedTransfer> transfers;
  TransfersLoaded(this.transfers);
}

class TransferTreeLoaded extends TransferState {
  final List<TransferTreeNode> tree;
  TransferTreeLoaded(this.tree);
}

class TransferSuccess extends TransferState {
  final String? message;
  TransferSuccess({this.message});
}

class TransferFailure extends TransferState {
  final String message;
  TransferFailure(this.message);
}

// Bloc
class TransferBloc extends Bloc<TransferEvent, TransferState> {
  final ITransferRepository _repository;

  TransferBloc(this._repository) : super(TransferInitial()) {
    on<SendFile>((event, emit) async {
      emit(TransferInProgress(0.0));
      try {
        await _repository.sendFile(event.file, receiverId: 'server');
        emit(TransferSuccess(message: 'Upload completed successfully.'));
      } catch (e) {
        emit(TransferFailure(e.toString()));
      }
    });

    on<FetchReceivedTransfers>((event, emit) async {
      emit(TransferInProgress(0.0));
      try {
        final transfers = await _repository.getReceivedTransfers();
        emit(TransfersLoaded(transfers));
      } catch (e) {
        emit(TransferFailure(e.toString()));
      }
    });

    on<DownloadReceivedFile>((event, emit) async {
      emit(TransferInProgress(0.0));
      try {
        await _repository.downloadFile(
          event.transfer.fileName,
          event.transfer.fileName,
        );
        emit(TransferSuccess(message: 'File downloaded successfully.'));
      } catch (e) {
        emit(TransferFailure(e.toString()));
      }
    });

    on<FetchTransferTree>((event, emit) async {
      emit(TransferInProgress(0.0));
      try {
        final tree = await _repository.getTransferTree(event.fileId);
        emit(TransferTreeLoaded(tree));
      } catch (e) {
        emit(TransferFailure(e.toString()));
      }
    });
  }
}

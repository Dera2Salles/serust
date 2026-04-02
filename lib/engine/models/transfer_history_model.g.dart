// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'transfer_history_model.dart';

// **************************************************************************
// TypeAdapterGenerator
// **************************************************************************

class TransferHistoryModelAdapter extends TypeAdapter<TransferHistoryModel> {
  @override
  final int typeId = 1;

  @override
  TransferHistoryModel read(BinaryReader reader) {
    final numOfFields = reader.readByte();
    final fields = <int, dynamic>{
      for (int i = 0; i < numOfFields; i++) reader.readByte(): reader.read(),
    };
    return TransferHistoryModel(
      id: fields[0] as String,
      fileName: fields[1] as String,
      fileSize: fields[2] as int,
      senderId: fields[3] as String,
      receiverId: fields[4] as String,
      timestamp: fields[5] as DateTime,
      status: fields[6] as String,
      isSent: fields[7] as bool,
      filePath: fields[8] as String?,
      maxShareCount: fields[9] as int?,
      shareCount: fields[10] as int,
    );
  }

  @override
  void write(BinaryWriter writer, TransferHistoryModel obj) {
    writer
      ..writeByte(11)
      ..writeByte(0)
      ..write(obj.id)
      ..writeByte(1)
      ..write(obj.fileName)
      ..writeByte(2)
      ..write(obj.fileSize)
      ..writeByte(3)
      ..write(obj.senderId)
      ..writeByte(4)
      ..write(obj.receiverId)
      ..writeByte(5)
      ..write(obj.timestamp)
      ..writeByte(6)
      ..write(obj.status)
      ..writeByte(7)
      ..write(obj.isSent)
      ..writeByte(8)
      ..write(obj.filePath)
      ..writeByte(9)
      ..write(obj.maxShareCount)
      ..writeByte(10)
      ..write(obj.shareCount);
  }

  @override
  int get hashCode => typeId.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is TransferHistoryModelAdapter &&
          runtimeType == other.runtimeType &&
          typeId == other.typeId;
}

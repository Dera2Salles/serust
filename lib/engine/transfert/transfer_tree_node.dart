import 'package:equatable/equatable.dart';

class TransferTreeNode extends Equatable {
  final String transferId;
  final String receiverId;
  final int transferLevel;
  final String status;
  final List<TransferTreeNode> children;

  const TransferTreeNode({
    required this.transferId,
    required this.receiverId,
    required this.transferLevel,
    required this.status,
    required this.children,
  });

  factory TransferTreeNode.fromJson(Map<String, dynamic> json) {
    return TransferTreeNode(
      transferId: json['transfer_id'].toString(),
      receiverId: json['receiver_id'].toString(),
      transferLevel: json['transfer_level'] ?? 0,
      status: json['status'] ?? 'unknown',
      children: (json['children'] as List? ?? [])
          .map(
            (child) => TransferTreeNode.fromJson(child as Map<String, dynamic>),
          )
          .toList(),
    );
  }

  @override
  List<Object?> get props => [
    transferId,
    receiverId,
    transferLevel,
    status,
    children,
  ];
}

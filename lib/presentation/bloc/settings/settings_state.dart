import 'package:equatable/equatable.dart';
import 'package:flutter/material.dart';

class SettingsState extends Equatable {
  final Color themeColor;
  final Locale locale;

  const SettingsState({required this.themeColor, required this.locale});

  SettingsState copyWith({Color? themeColor, Locale? locale}) {
    return SettingsState(
      themeColor: themeColor ?? this.themeColor,
      locale: locale ?? this.locale,
    );
  }

  @override
  List<Object> get props => [themeColor, locale];
}

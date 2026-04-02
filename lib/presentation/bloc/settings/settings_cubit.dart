import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:arosaina/engine/services/interfaces/i_settings_service.dart';

import 'settings_state.dart';

class SettingsCubit extends Cubit<SettingsState> {
  final ISettingsService _settingsService;

  SettingsCubit(this._settingsService)
    : super(
        SettingsState(
          themeColor: Color(_settingsService.themeColor),
          locale: Locale(_settingsService.appLocale),
        ),
      );

  void setThemeColor(Color color) {
    _settingsService.setThemeColor(color.toARGB32());
    emit(state.copyWith(themeColor: color));
  }

  void setLocale(Locale locale) {
    _settingsService.setAppLocale(locale.languageCode);
    emit(state.copyWith(locale: locale));
  }
}

import 'interfaces/i_settings_service.dart';
import 'interfaces/i_db_service.dart';

class SettingsServiceImpl implements ISettingsService {
  final IDbService _dbService;

  SettingsServiceImpl(this._dbService);

  @override
  bool get hasSeenOnboarding =>
      _dbService.settingsBox.get('hasSeenOnboarding', defaultValue: false);

  @override
  Future<void> setHasSeenOnboarding(bool value) =>
      _dbService.settingsBox.put('hasSeenOnboarding', value);

  @override
  int get themeColor =>
      _dbService.settingsBox.get('themeColor', defaultValue: 0xFF7B42F6); // Default Purple

  @override
  Future<void> setThemeColor(int value) =>
      _dbService.settingsBox.put('themeColor', value);

  @override
  String get appLocale =>
      _dbService.settingsBox.get('appLocale', defaultValue: 'en');

  @override
  Future<void> setAppLocale(String value) =>
      _dbService.settingsBox.put('appLocale', value);

  @override
  String get apiBaseUrl =>
      _dbService.settingsBox.get('apiBaseUrl', defaultValue: 'http://localhost:8080');

  @override
  Future<void> setApiBaseUrl(String value) =>
      _dbService.settingsBox.put('apiBaseUrl', value);
}

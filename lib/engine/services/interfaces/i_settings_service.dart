abstract class ISettingsService {
  bool get hasSeenOnboarding;
  Future<void> setHasSeenOnboarding(bool value);

  int get themeColor;
  Future<void> setThemeColor(int value);

  String get appLocale;
  Future<void> setAppLocale(String value);

  String get apiBaseUrl;
  Future<void> setApiBaseUrl(String value);
}

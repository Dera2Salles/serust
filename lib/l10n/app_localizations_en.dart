// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get settingsTitle => 'Settings';

  @override
  String get themeTitle => 'Theme';

  @override
  String get themeSubtitle => 'Choose your preferred color';

  @override
  String get languageTitle => 'Language';

  @override
  String get languageSubtitle => 'Select app language';

  @override
  String get languageEnglish => 'English';

  @override
  String get languageFrench => 'French';

  @override
  String get homeTitle => 'Home';

  @override
  String get sendTitle => 'Send';

  @override
  String get receiveTitle => 'Receive';

  @override
  String get historyTitle => 'History';

  @override
  String get recentTransfers => 'Recent Transfers';

  @override
  String get shareConfidence => 'Share with confidence';
}

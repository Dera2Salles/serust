// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for French (`fr`).
class AppLocalizationsFr extends AppLocalizations {
  AppLocalizationsFr([String locale = 'fr']) : super(locale);

  @override
  String get settingsTitle => 'Paramètres';

  @override
  String get themeTitle => 'Thème';

  @override
  String get themeSubtitle => 'Choisissez votre couleur préférée';

  @override
  String get languageTitle => 'Langue';

  @override
  String get languageSubtitle => 'Sélectionnez la langue de l\'application';

  @override
  String get languageEnglish => 'Anglais';

  @override
  String get languageFrench => 'Français';

  @override
  String get homeTitle => 'Accueil';

  @override
  String get sendTitle => 'Envoyer';

  @override
  String get receiveTitle => 'Recevoir';

  @override
  String get historyTitle => 'Historique';

  @override
  String get recentTransfers => 'Transferts récents';

  @override
  String get shareConfidence => 'Partagez en toute confiance';
}

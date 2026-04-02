import 'package:arosaina/core/init.dart';
import 'package:arosaina/core/injection.dart';
import 'package:arosaina/engine/services/interfaces/i_auth_service.dart';
import 'package:arosaina/engine/services/interfaces/i_settings_service.dart';
// DbService init call is in setupLocator actually? No, main calls Init.execute().
// Let's check main content again.

import 'package:arosaina/presentation/screens/auth/login_screen.dart';
import 'package:arosaina/presentation/screens/home/home_screen.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:arosaina/l10n/app_localizations.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'presentation/bloc/connection/connection_bloc.dart';
import 'presentation/bloc/settings/settings_cubit.dart';
import 'presentation/bloc/settings/settings_state.dart';
import 'presentation/bloc/transfer/transfer_bloc.dart';
import 'presentation/screens/onboarding/onboarding_screen.dart';
import 'presentation/theme/app_theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await setupLocator();
  await Init.execute();

  runApp(const AroSaina());
}

class AroSaina extends StatelessWidget {
  const AroSaina({super.key});

  @override
  Widget build(BuildContext context) {
    final settingsService = sl<ISettingsService>();
    final hasSeenOnboarding = settingsService.hasSeenOnboarding;
    final isLoggedIn = sl<IAuthService>().getCurrentUser() != null;

    Widget initialScreen;
    if (!hasSeenOnboarding) {
      initialScreen = const OnboardingScreen();
    } else if (isLoggedIn) {
      initialScreen = const HomeScreen();
    } else {
      initialScreen = const LoginScreen();
    }

    return MultiBlocProvider(
      providers: [
        BlocProvider(create: (context) => sl<ConnectionBloc>()),
        BlocProvider(create: (context) => sl<TransferBloc>()),
        BlocProvider(
          create: (context) => SettingsCubit(sl<ISettingsService>()),
        ),
      ],
      child: BlocBuilder<SettingsCubit, SettingsState>(
        builder: (context, state) {
          return MaterialApp(
            title: 'AroSaina',
            theme: AppTheme.lightTheme(),
            darkTheme: AppTheme.darkTheme(),
            themeMode: ThemeMode.light,
            locale: state.locale,
            localizationsDelegates: const [
              AppLocalizations.delegate,
              GlobalMaterialLocalizations.delegate,
              GlobalWidgetsLocalizations.delegate,
              GlobalCupertinoLocalizations.delegate,
            ],
            supportedLocales: const [Locale('en'), Locale('fr')],
            home: initialScreen,
            debugShowCheckedModeBanner: false,
          );
        },
      ),
    );
  }
}

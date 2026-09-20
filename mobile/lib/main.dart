import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'theme/cyber_theme.dart';
import 'screens/home_screen.dart';
import 'widgets/floating_overlay_widget.dart';

/// Entry point for the floating overlay window
@pragma("vm:entry-point")
void overlayMain() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(
    const MaterialApp(
      debugShowCheckedModeBanner: false,
      home: FloatingOverlayWidget(),
    ),
  );
}

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent,
      statusBarIconBrightness: Brightness.light,
      systemNavigationBarColor: CyberTheme.bgCosmic,
      systemNavigationBarIconBrightness: Brightness.light,
    ),
  );
  runApp(const TaskNoteApp());
}

class TaskNoteApp extends StatelessWidget {
  const TaskNoteApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Task Note & Time Keeping',
      debugShowCheckedModeBanner: false,
      theme: CyberTheme.darkTheme,
      home: const HomeScreen(),
    );
  }
}

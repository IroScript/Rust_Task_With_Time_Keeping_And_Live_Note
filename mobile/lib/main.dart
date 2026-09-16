import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'theme/cyber_theme.dart';
import 'screens/home_screen.dart';

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

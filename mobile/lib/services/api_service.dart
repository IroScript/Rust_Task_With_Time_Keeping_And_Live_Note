import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';
import '../models/task_card.dart';

class ApiService {
  static const String defaultBaseUrl = 'http://127.0.0.1:3000';
  String baseUrl;

  ApiService({this.baseUrl = defaultBaseUrl});

  Future<void> setBaseUrl(String url) async {
    baseUrl = url;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('api_base_url', url);
  }

  Future<void> loadSavedBaseUrl() async {
    final prefs = await SharedPreferences.getInstance();
    final saved = prefs.getString('api_base_url');
    if (saved != null && saved.isNotEmpty) {
      baseUrl = saved;
    }
  }

  /// Check server health
  Future<bool> checkHealth() async {
    try {
      final response = await http
          .get(Uri.parse('$baseUrl/health'))
          .timeout(const Duration(seconds: 3));
      return response.statusCode == 200 && response.body.contains('OK');
    } catch (_) {
      return false;
    }
  }

  /// Fetch cards from Axum backend
  Future<List<TaskCard>> fetchCards({int page = 1, int perPage = 50}) async {
    try {
      final response = await http
          .get(Uri.parse('$baseUrl/api/cards?page=$page&per_page=$perPage'))
          .timeout(const Duration(seconds: 4));

      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        final List list = data['cards'] ?? [];
        return list.map((c) => TaskCard.fromAxumJson(c)).toList();
      }
    } catch (e) {
      // Fallback handled in provider
    }
    return [];
  }

  /// Create card on Axum backend
  Future<TaskCard?> createCard(String title) async {
    try {
      final response = await http
          .post(
            Uri.parse('$baseUrl/api/cards'),
            headers: {'Content-Type': 'application/json'},
            body: jsonEncode({'title': title}),
          )
          .timeout(const Duration(seconds: 4));

      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        return TaskCard.fromAxumJson(data);
      }
    } catch (_) {}
    return null;
  }

  /// Get lines for virtual scrolling from Axum
  Future<List<String>> fetchCardLines(
    String cardId, {
    int startLine = 1,
    int limit = 50,
  }) async {
    try {
      final response = await http
          .get(Uri.parse(
              '$baseUrl/api/cards/$cardId/lines?start_line=$startLine&limit=$limit'))
          .timeout(const Duration(seconds: 4));

      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        final List lines = data['lines'] ?? [];
        return lines.map((l) => l['line_text'].toString()).toList();
      }
    } catch (_) {}
    return [];
  }

  /// Update single line in card
  Future<bool> updateLine(
    String cardId,
    int lineNumber,
    String lineText,
  ) async {
    try {
      final response = await http
          .put(
            Uri.parse('$baseUrl/api/cards/$cardId/lines/$lineNumber'),
            headers: {'Content-Type': 'application/json'},
            body: jsonEncode({'line_text': lineText}),
          )
          .timeout(const Duration(seconds: 4));

      return response.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Local Storage persistence
  Future<void> saveLocalCards(List<TaskCard> cards) async {
    final prefs = await SharedPreferences.getInstance();
    final jsonList = cards.map((c) => c.toLocalJson()).toList();
    await prefs.setString('cached_cards', jsonEncode(jsonList));
  }

  Future<List<TaskCard>> loadLocalCards() async {
    final prefs = await SharedPreferences.getInstance();
    final jsonStr = prefs.getString('cached_cards');
    if (jsonStr != null && jsonStr.isNotEmpty) {
      try {
        final List decoded = jsonDecode(jsonStr);
        return decoded.map((c) => TaskCard.fromLocalJson(c)).toList();
      } catch (_) {}
    }
    return [];
  }
}

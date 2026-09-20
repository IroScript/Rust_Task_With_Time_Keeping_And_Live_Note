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
        final List list = data is Map ? (data['cards'] ?? []) : (data is List ? data : []);
        return list.map((c) => TaskCard.fromAxumJson(c)).toList();
      }
    } catch (e) {
      // Handled in caller
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

      if (response.statusCode == 200 || response.statusCode == 201) {
        final data = jsonDecode(response.body);
        return TaskCard.fromAxumJson(data);
      }
    } catch (_) {}
    return null;
  }

  /// Fetch card metadata (/api/cards/:id/meta)
  Future<Map<String, dynamic>?> fetchCardMetadata(String cardId) async {
    try {
      final response = await http
          .get(Uri.parse('$baseUrl/api/cards/$cardId/meta'))
          .timeout(const Duration(seconds: 4));

      if (response.statusCode == 200) {
        return jsonDecode(response.body);
      }
    } catch (_) {}
    return null;
  }

  /// Get lines for virtual scrolling from Axum
  Future<List<Map<String, dynamic>>> fetchCardLineEntries(
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
        final List list = data is List ? data : (data is Map ? (data['lines'] ?? []) : []);
        return list.map((l) => {
          'line_number': l['line_number'] ?? 0,
          'line_text': l['line_text']?.toString() ?? '',
        }).toList();
      }
    } catch (_) {}
    return [];
  }

  /// Update single line in card (/api/cards/:id/lines/:line_number)
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

  /// Local Storage persistence for cards
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

  /// Save App Settings (local cache + Axum cloud upsert)
  Future<void> saveAppSettings(
    Map<String, dynamic> settings, {
    String userId = '9969c846e8e1642bcefa356398644f3b',
  }) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('app_settings', jsonEncode(settings));

    // Also sync to Axum backend: /api/users/{user_id}/settings
    try {
      await http
          .post(
            Uri.parse('$baseUrl/api/users/$userId/settings'),
            headers: {'Content-Type': 'application/json'},
            body: jsonEncode({'settings_data': settings}),
          )
          .timeout(const Duration(seconds: 3));
    } catch (_) {}
  }

  /// Load App Settings (Axum cloud with local fallback)
  Future<Map<String, dynamic>> loadAppSettings({
    String userId = '9969c846e8e1642bcefa356398644f3b',
  }) async {
    final prefs = await SharedPreferences.getInstance();
    final str = prefs.getString('app_settings');
    Map<String, dynamic> local = {};
    if (str != null && str.isNotEmpty) {
      try {
        local = jsonDecode(str);
      } catch (_) {}
    }

    try {
      final response = await http
          .get(Uri.parse('$baseUrl/api/users/$userId/settings'))
          .timeout(const Duration(seconds: 3));
      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        if (data is Map && data['settings_data'] != null) {
          final settingsData = data['settings_data'];
          if (settingsData is String) {
            return jsonDecode(settingsData);
          } else if (settingsData is Map) {
            return Map<String, dynamic>.from(settingsData);
          }
        }
      }
    } catch (_) {}

    return local;
  }
}

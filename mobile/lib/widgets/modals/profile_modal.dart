import 'package:flutter/material.dart';
import '../../theme/cyber_theme.dart';

class ProfileModal extends StatefulWidget {
  final Map<String, String> userProfile;
  final bool isBackendOnline;
  final Function(Map<String, String> updatedProfile) onSave;

  const ProfileModal({
    super.key,
    required this.userProfile,
    required this.isBackendOnline,
    required this.onSave,
  });

  @override
  State<ProfileModal> createState() => _ProfileModalState();
}

class _ProfileModalState extends State<ProfileModal> {
  late TextEditingController _nameController;
  late TextEditingController _emailController;
  late TextEditingController _countryController;
  late TextEditingController _companyController;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: widget.userProfile['name'] ?? 'Irak');
    _emailController = TextEditingController(text: widget.userProfile['email'] ?? 'irak@iroscript.org');
    _countryController = TextEditingController(text: widget.userProfile['country'] ?? 'Bangladesh');
    _companyController = TextEditingController(text: widget.userProfile['company'] ?? 'IroScript');
  }

  @override
  void dispose() {
    _nameController.dispose();
    _emailController.dispose();
    _countryController.dispose();
    _companyController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.symmetric(horizontal: 18, vertical: 30),
      child: Container(
        decoration: BoxDecoration(
          color: const Color(0xF70D121F),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: CyberTheme.neonCyan, width: 1.5),
          boxShadow: [
            BoxShadow(
              color: CyberTheme.neonCyan.withValues(alpha: 0.25),
              blurRadius: 18,
            ),
          ],
        ),
        padding: const EdgeInsets.all(16),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Header
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Row(
                    children: [
                      Icon(Icons.person, color: CyberTheme.neonCyan, size: 18),
                      SizedBox(width: 8),
                      Text(
                        'USER PROFILE & SYNC',
                        style: TextStyle(
                          color: CyberTheme.neonCyan,
                          fontSize: 13,
                          fontWeight: FontWeight.bold,
                          fontFamily: 'monospace',
                          letterSpacing: 1.0,
                        ),
                      ),
                    ],
                  ),
                  IconButton(
                    icon: const Icon(Icons.close, color: Colors.white, size: 18),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ],
              ),
              const Divider(color: CyberTheme.borderSubtle),

              // Backend sync status
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                decoration: BoxDecoration(
                  color: widget.isBackendOnline
                      ? CyberTheme.neonLime.withValues(alpha: 0.12)
                      : Colors.redAccent.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(
                    color: widget.isBackendOnline ? CyberTheme.neonLime : Colors.redAccent,
                    width: 0.8,
                  ),
                ),
                child: Row(
                  children: [
                    Icon(
                      widget.isBackendOnline ? Icons.cloud_done : Icons.cloud_off,
                      color: widget.isBackendOnline ? CyberTheme.neonLime : Colors.redAccent,
                      size: 14,
                    ),
                    const SizedBox(width: 8),
                    Text(
                      widget.isBackendOnline
                          ? 'Axum Cloud Sync: ONLINE'
                          : 'Axum Cloud Sync: OFFLINE (Local Memory Active)',
                      style: TextStyle(
                        color: widget.isBackendOnline ? CyberTheme.neonLime : Colors.redAccent,
                        fontSize: 10,
                        fontWeight: FontWeight.bold,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 14),

              _buildField('Full Name', _nameController),
              const SizedBox(height: 10),
              _buildField('Email', _emailController),
              const SizedBox(height: 10),
              _buildField('Country', _countryController),
              const SizedBox(height: 10),
              _buildField('Organization / Company', _companyController),

              const SizedBox(height: 18),
              ElevatedButton.icon(
                style: ElevatedButton.styleFrom(
                  backgroundColor: CyberTheme.bgCardActive,
                  foregroundColor: CyberTheme.neonCyan,
                  side: const BorderSide(color: CyberTheme.neonCyan, width: 1.2),
                  padding: const EdgeInsets.symmetric(vertical: 12),
                ),
                icon: const Icon(Icons.save, size: 16),
                label: const Text(
                  'SAVE PROFILE',
                  style: TextStyle(fontFamily: 'monospace', fontWeight: FontWeight.bold),
                ),
                onPressed: () {
                  widget.onSave({
                    'name': _nameController.text.trim(),
                    'email': _emailController.text.trim(),
                    'country': _countryController.text.trim(),
                    'company': _companyController.text.trim(),
                  });
                  Navigator.of(context).pop();
                },
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildField(String label, TextEditingController controller) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label.toUpperCase(),
          style: const TextStyle(
            color: CyberTheme.textSecondary,
            fontSize: 10,
            fontWeight: FontWeight.bold,
            fontFamily: 'monospace',
          ),
        ),
        const SizedBox(height: 4),
        TextField(
          controller: controller,
          style: const TextStyle(color: Colors.white, fontSize: 12, fontFamily: 'monospace'),
          decoration: InputDecoration(
            isDense: true,
            filled: true,
            fillColor: Colors.black45,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(6),
              borderSide: const BorderSide(color: CyberTheme.borderSubtle),
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(6),
              borderSide: const BorderSide(color: CyberTheme.neonCyan),
            ),
          ),
        ),
      ],
    );
  }
}

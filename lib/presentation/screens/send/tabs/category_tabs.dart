import 'package:arosaina/presentation/common/responsive.dart';
import 'package:arosaina/presentation/services/media_service.dart';
import 'package:arosaina/presentation/services/selectable_item.dart';
import 'package:arosaina/presentation/theme/app_theme.dart';
import 'package:flutter/material.dart';
import 'package:arosaina/core/injection.dart';

class CategoryTabs extends StatefulWidget {
  final Function(SelectableItem) onFileSelected;
  final List<SelectableItem> selectedFiles;

  const CategoryTabs({
    super.key,
    required this.onFileSelected,
    required this.selectedFiles,
  });

  @override
  State<CategoryTabs> createState() => _CategoryTabsState();
}

class _CategoryTabsState extends State<CategoryTabs> {
  String _selectedCategory = 'Images';
  final MediaService _mediaService = sl<MediaService>();
  List<SelectableItem> _items = [];
  bool _isLoading = true;

  final List<Map<String, dynamic>> _categories = [
    {
      'name': 'Images',
      'icon': Icons.image_rounded,
      'color': Colors.teal,
      'type': MediaType.image,
    },
    {
      'name': 'Videos',
      'icon': Icons.videocam_rounded,
      'color': Colors.blue,
      'type': MediaType.video,
    },
    {
      'name': 'Audio',
      'icon': Icons.music_note_rounded,
      'color': Colors.purple,
      'type': MediaType.audio,
    },
    {
      'name': 'Apps',
      'icon': Icons.android_rounded,
      'color': Colors.green,
      'type': MediaType.app,
    },
    {
      'name': 'Docs',
      'icon': Icons.description_rounded,
      'color': Colors.orange,
      'type': MediaType.file,
    },
  ];

  @override
  void initState() {
    super.initState();
    _loadCategory(_selectedCategory);
  }

  Future<void> _loadCategory(String categoryName) async {
    setState(() => _isLoading = true);
    try {
      final cat = _categories.firstWhere((c) => c['name'] == categoryName);
      final items = await _mediaService.getMedia(cat['type'] as MediaType);
      setState(() {
        _items = items;
        _isLoading = false;
      });
    } catch (e) {
      if (mounted) {
        setState(() => _isLoading = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Column(
      children: [
        // Category Selector
        SizedBox(
          height: context.rs(100),
          child: ListView.builder(
            scrollDirection: Axis.horizontal,
            padding: EdgeInsets.symmetric(horizontal: context.rs(24)),
            itemCount: _categories.length,
            itemBuilder: (context, index) {
              final cat = _categories[index];
              final isSelected = _selectedCategory == cat['name'];
              return GestureDetector(
                onTap: () {
                  setState(() => _selectedCategory = cat['name'] as String);
                  _loadCategory(_selectedCategory);
                },
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 200),
                  margin: EdgeInsets.only(
                    right: context.rs(16),
                    top: context.rs(10),
                    bottom: context.rs(10),
                  ),
                  width: context.rs(72),
                  decoration: BoxDecoration(
                    color: isSelected
                        ? cat['color'] as Color
                        : (isDark ? AppTheme.neutral800 : Colors.white),
                    borderRadius: BorderRadius.circular(context.rs(20)),
                    border: Border.all(
                      color: isSelected
                          ? Colors.transparent
                          : (isDark
                                ? AppTheme.neutral700
                                : AppTheme.neutral200),
                    ),
                    boxShadow: isSelected
                        ? [
                            BoxShadow(
                              color: (cat['color'] as Color).withValues(
                                alpha: 0.3,
                              ),
                              blurRadius: 8,
                              offset: const Offset(0, 4),
                            ),
                          ]
                        : null,
                  ),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(
                        cat['icon'] as IconData,
                        color: isSelected
                            ? Colors.white
                            : (isDark
                                  ? AppTheme.neutral300
                                  : AppTheme.neutral500),
                        size: 24,
                      ),
                      SizedBox(height: context.rs(4)),
                      Text(
                        cat['name'] as String,
                        style: TextStyle(
                          color: isSelected
                              ? Colors.white
                              : (isDark
                                    ? AppTheme.neutral100
                                    : AppTheme.neutral700),
                          fontSize: context.rf(10),
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                ),
              );
            },
          ),
        ),

        // Items Grid
        Expanded(
          child: _isLoading
              ? Center(
                  child: CircularProgressIndicator(
                    color: Theme.of(context).primaryColor,
                  ),
                )
              : _items.isEmpty
              ? Center(
                  child: Text(
                    "No items found",
                    style: TextStyle(color: AppTheme.neutral500),
                  ),
                )
              : GridView.builder(
                  padding: EdgeInsets.all(context.rs(24)),
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: 3,
                    crossAxisSpacing: context.rs(12),
                    mainAxisSpacing: context.rs(12),
                    childAspectRatio: 0.8,
                  ),
                  itemCount: _items.length,
                  itemBuilder: (context, index) {
                    final item = _items[index];
                    final isSelected = widget.selectedFiles.any(
                      (f) => f.path == item.path,
                    );
                    return _CategoryItem(
                      item: item,
                      isSelected: isSelected,
                      onTap: () => widget.onFileSelected(item),
                      isDark: isDark,
                    );
                  },
                ),
        ),
      ],
    );
  }
}

class _CategoryItem extends StatelessWidget {
  final SelectableItem item;
  final bool isSelected;
  final VoidCallback onTap;
  final bool isDark;

  const _CategoryItem({
    required this.item,
    required this.isSelected,
    required this.onTap,
    required this.isDark,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          color: isDark ? AppTheme.neutral800 : Colors.white,
          borderRadius: BorderRadius.circular(context.rs(16)),
          border: Border.all(
            color: isSelected
                ? Theme.of(context).primaryColor
                : (isDark ? AppTheme.neutral700 : AppTheme.neutral200),
            width: isSelected ? 2 : 1,
          ),
        ),
        child: Stack(
          children: [
            Column(
              children: [
                Expanded(
                  child: ClipRRect(
                    borderRadius: const BorderRadius.vertical(
                      top: Radius.circular(15),
                    ),
                    child: Container(
                      width: double.infinity,
                      color: isDark ? AppTheme.neutral700 : AppTheme.neutral100,
                      child: _buildThumbnail(),
                    ),
                  ),
                ),
                Padding(
                  padding: EdgeInsets.all(context.rs(8)),
                  child: Text(
                    item.name,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: context.rf(10),
                      fontWeight: FontWeight.bold,
                      color: isDark ? Colors.white : AppTheme.neutral900,
                    ),
                  ),
                ),
              ],
            ),
            if (isSelected)
              Positioned(
                top: 4,
                right: 4,
                child: Container(
                  padding: const EdgeInsets.all(2),
                  decoration: BoxDecoration(
                    color: Theme.of(context).primaryColor,
                    shape: BoxShape.circle,
                  ),
                  child: const Icon(Icons.check, size: 12, color: Colors.white),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildThumbnail() {
    if (item.thumbnail != null) {
      return Image.memory(item.thumbnail!, fit: BoxFit.cover);
    }
    return Icon(
      Icons.insert_drive_file_rounded,
      color: AppTheme.neutral400,
      size: 32,
    );
  }
}

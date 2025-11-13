//! Marketplace and UI Themes Module
//! 
//! Module for plugin marketplace search and UI theme management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marketplace {
    pub plugins: Vec<Plugin>,
    pub themes: Vec<Theme>,
    pub search_index: HashMap<String, Vec<String>>, // keyword -> [item_ids]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f32, // 1-5
    pub compatibility: Vec<String>, // supported versions
    pub download_url: String,
    pub license: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
    pub colors: ThemeColors,
    pub components: Vec<String>, // which UI components the theme affects
    pub preview_image: Option<String>,
    pub download_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub surface: String,
    pub error: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_on_primary: String,
    pub text_on_surface: String,
    pub text_on_error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearch {
    pub query: String,
    pub filters: SearchFilters,
    pub sort_by: SortOption,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub rating_min: Option<u8>,
    pub downloads_min: Option<u64>,
    pub compatible_with: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOption {
    Relevance,
    Downloads,
    Rating,
    Newest,
    Name,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResult {
    Plugin(Plugin),
    Theme(Theme),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManager {
    pub current_theme: String,
    pub available_themes: HashMap<String, Theme>,
    pub theme_directory: String,
}

impl Marketplace {
    pub fn new() -> Self {
        Self {
            plugins: vec![],
            themes: vec![],
            search_index: HashMap::new(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Plugin) {
        self.plugins.push(plugin);
        self.update_search_index();
    }

    pub fn add_theme(&mut self, theme: Theme) {
        self.themes.push(theme);
        self.update_search_index();
    }

    pub fn search(&self, query: &str, filters: &SearchFilters, sort_by: &SortOption) -> Vec<SearchResult> {
        let mut results = vec![];

        // Search plugins
        for plugin in &self.plugins {
            if self.matches_query_and_filters(plugin, query, filters) {
                results.push(SearchResult::Plugin(plugin.clone()));
            }
        }

        // Search themes
        for theme in &self.themes {
            if self.matches_query_and_filters(theme, query, filters) {
                results.push(SearchResult::Theme(theme.clone()));
            }
        }

        // Sort results
        self.sort_results(&mut results, sort_by);

        results
    }

    fn matches_query_and_filters<T: Searchable>(&self, item: &T, query: &str, filters: &SearchFilters) -> bool {
        // Check if query matches
        if !item.matches_query(query) {
            return false;
        }

        // Apply filters
        if let Some(ref category) = filters.category {
            if !item.category().eq_ignore_ascii_case(category) {
                return false;
            }
        }

        if let Some(rating_min) = filters.rating_min {
            if item.rating() < rating_min as f32 {
                return false;
            }
        }

        if let Some(downloads_min) = filters.downloads_min {
            if item.downloads() < downloads_min {
                return false;
            }
        }

        if !filters.tags.is_empty() {
            for tag in &filters.tags {
                if !item.tags().iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                    return false; // All tags must match
                }
            }
        }

        true
    }

    fn sort_results(&self, results: &mut Vec<SearchResult>, sort_by: &SortOption) {
        match sort_by {
            SortOption::Relevance => {
                // For simplicity, we'll sort by rating then downloads
                results.sort_by(|a, b| {
                    let rating_a = match a {
                        SearchResult::Plugin(p) => p.rating,
                        SearchResult::Theme(t) => t.rating,
                    };
                    let rating_b = match b {
                        SearchResult::Plugin(p) => p.rating,
                        SearchResult::Theme(t) => t.rating,
                    };
                    
                    rating_b.partial_cmp(&rating_a).unwrap().then_with(|| {
                        let downloads_a = match a {
                            SearchResult::Plugin(p) => p.downloads,
                            SearchResult::Theme(t) => t.downloads,
                        };
                        let downloads_b = match b {
                            SearchResult::Plugin(p) => p.downloads,
                            SearchResult::Theme(t) => t.downloads,
                        };
                        downloads_b.cmp(&downloads_a)
                    })
                });
            }
            SortOption::Downloads => {
                results.sort_by(|a, b| {
                    let downloads_a = match a {
                        SearchResult::Plugin(p) => p.downloads,
                        SearchResult::Theme(t) => t.downloads,
                    };
                    let downloads_b = match b {
                        SearchResult::Plugin(p) => p.downloads,
                        SearchResult::Theme(t) => t.downloads,
                    };
                    downloads_b.cmp(&downloads_a)
                });
            }
            SortOption::Rating => {
                results.sort_by(|a, b| {
                    let rating_a = match a {
                        SearchResult::Plugin(p) => p.rating,
                        SearchResult::Theme(t) => t.rating,
                    };
                    let rating_b = match b {
                        SearchResult::Plugin(p) => p.rating,
                        SearchResult::Theme(t) => t.rating,
                    };
                    rating_b.partial_cmp(&rating_a).unwrap()
                });
            }
            SortOption::Newest => {
                // Sort by date - newest first
                results.sort_by(|a, b| {
                    let date_a = match a {
                        SearchResult::Plugin(p) => &p.created_at,
                        SearchResult::Theme(t) => &t.created_at,
                    };
                    let date_b = match b {
                        SearchResult::Plugin(p) => &p.created_at,
                        SearchResult::Theme(t) => &t.created_at,
                    };
                    date_b.cmp(date_a)
                });
            }
            SortOption::Name => {
                results.sort_by(|a, b| {
                    let name_a = match a {
                        SearchResult::Plugin(p) => &p.name,
                        SearchResult::Theme(t) => &t.name,
                    };
                    let name_b = match b {
                        SearchResult::Plugin(p) => &p.name,
                        SearchResult::Theme(t) => &t.name,
                    };
                    name_a.cmp(name_b)
                });
            }
        }
    }

    fn update_search_index(&mut self) {
        // Build search index - in a real implementation this would be more sophisticated
        let mut index = HashMap::new();
        
        for plugin in &self.plugins {
            for tag in &plugin.tags {
                index.entry(tag.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(format!("plugin_{}", plugin.id));
            }
            index.entry(plugin.name.to_lowercase())
                .or_insert_with(Vec::new)
                .push(format!("plugin_{}", plugin.id));
        }
        
        for theme in &self.themes {
            for tag in &theme.tags {
                index.entry(tag.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(format!("theme_{}", theme.id));
            }
            index.entry(theme.name.to_lowercase())
                .or_insert_with(Vec::new)
                .push(format!("theme_{}", theme.id));
        }
        
        self.search_index = index;
    }
}

trait Searchable {
    fn matches_query(&self, query: &str) -> bool;
    fn category(&self) -> &str;
    fn rating(&self) -> f32;
    fn downloads(&self) -> u64;
    fn tags(&self) -> &Vec<String>;
}

impl Searchable for Plugin {
    fn matches_query(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(&query.to_lowercase()) ||
        self.description.to_lowercase().contains(&query.to_lowercase()) ||
        self.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
    }

    fn category(&self) -> &str {
        &self.category
    }

    fn rating(&self) -> f32 {
        self.rating
    }

    fn downloads(&self) -> u64 {
        self.downloads
    }

    fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}

impl Searchable for Theme {
    fn matches_query(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(&query.to_lowercase()) ||
        self.description.to_lowercase().contains(&query.to_lowercase()) ||
        self.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
    }

    fn category(&self) -> &str {
        "theme" // Themes don't have categories in this model
    }

    fn rating(&self) -> f32 {
        self.rating
    }

    fn downloads(&self) -> u64 {
        self.downloads
    }

    fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}

impl ThemeManager {
    pub fn new(theme_dir: String) -> Self {
        Self {
            current_theme: "default".to_string(),
            available_themes: HashMap::new(),
            theme_directory: theme_dir,
        }
    }

    pub fn load_themes(&mut self) -> Result<()> {
        // In a real implementation, this would scan the theme directory
        // and load theme files
        
        // For simulation, we'll add some default themes
        let default_theme = Theme {
            id: "default".to_string(),
            name: "Default Theme".to_string(),
            version: "1.0.0".to_string(),
            description: "The default theme for Kandil Code".to_string(),
            author: "Kandil Team".to_string(),
            tags: vec!["default".to_string(), "light".to_string()],
            downloads: 0,
            rating: 4.5,
            colors: ThemeColors {
                primary: "#1976D2".to_string(),
                secondary: "#1976D2".to_string(),
                background: "#FFFFFF".to_string(),
                surface: "#F5F5F5".to_string(),
                error: "#D32F2F".to_string(),
                text_primary: "#212121".to_string(),
                text_secondary: "#757575".to_string(),
                text_on_primary: "#FFFFFF".to_string(),
                text_on_surface: "#212121".to_string(),
                text_on_error: "#FFFFFF".to_string(),
            },
            components: vec![
                "editor".to_string(),
                "sidebar".to_string(),
                "toolbar".to_string(),
                "dialog".to_string(),
            ],
            preview_image: None,
            download_url: "".to_string(),
            created_at: "2023-01-01".to_string(),
            updated_at: "2023-01-01".to_string(),
        };
        
        let dark_theme = Theme {
            id: "dark".to_string(),
            name: "Dark Theme".to_string(),
            version: "1.0.0".to_string(),
            description: "A dark theme for Kandil Code".to_string(),
            author: "Kandil Team".to_string(),
            tags: vec!["dark".to_string(), "night".to_string()],
            downloads: 0,
            rating: 4.7,
            colors: ThemeColors {
                primary: "#90CAF9".to_string(),
                secondary: "#90CAF9".to_string(),
                background: "#121212".to_string(),
                surface: "#1E1E1E".to_string(),
                error: "#EF5350".to_string(),
                text_primary: "#FFFFFF".to_string(),
                text_secondary: "#BDBDBD".to_string(),
                text_on_primary: "#000000".to_string(),
                text_on_surface: "#FFFFFF".to_string(),
                text_on_error: "#000000".to_string(),
            },
            components: vec![
                "editor".to_string(),
                "sidebar".to_string(),
                "toolbar".to_string(),
                "dialog".to_string(),
            ],
            preview_image: None,
            download_url: "".to_string(),
            created_at: "2023-01-01".to_string(),
            updated_at: "2023-01-01".to_string(),
        };
        
        self.available_themes.insert("default".to_string(), default_theme);
        self.available_themes.insert("dark".to_string(), dark_theme);
        
        Ok(())
    }

    pub fn apply_theme(&mut self, theme_id: &str) -> Result<()> {
        if self.available_themes.contains_key(theme_id) {
            self.current_theme = theme_id.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Theme {} not found", theme_id))
        }
    }

    pub fn get_current_theme(&self) -> Option<&Theme> {
        self.available_themes.get(&self.current_theme)
    }

    pub fn get_available_themes(&self) -> Vec<&Theme> {
        self.available_themes.values().collect()
    }

    pub fn create_custom_theme(&mut self, theme: Theme) -> Result<()> {
        self.available_themes.insert(theme.id.clone(), theme);
        Ok(())
    }
}
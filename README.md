# Medow - Mediathek Downloader

Search and downloads media of german public television services using the _Mediathekviewweb_ API. 

## Features

- [x] Basic search by title and or topic
- [ ] Pagination
- [ ] Download selected media entries
- [ ] use title and a numbered prefix as the destination filename instead of original filename 
- [ ] Settings (at least to choose a default destination folder)
- [ ] persist app state - remember last _x_ search terms and be able to repeat a search
- [ ] Resume failed or aborted downloads
- [ ] tbd...



## Development

The project follows a standard Rust/Dioxus structure with multiple modules for different functionalities.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
│  ├─ favicon.ico # Application favicon
│  ├─ main.css # Main stylesheet
│  └─ pico.blue.min.css # CSS framework for styling
├─ src/
│  ├─ main.rs # Entry point to your application
│  ├─ search_logic.rs # Logic for searching media
│  ├─ search_view.rs # UI components for search interface
│  ├─ pagination.rs # Pagination logic
│  └─ utils.rs # Utility functions
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
└─ Dioxus.toml # Configuration for Dioxus application
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

### Project Structure Explanation

- **main.rs**: Contains the main application entry point and top-level components
- **search_logic.rs**: Handles all search-related functionality using the Mediathekviewweb API
- **search_view.rs**: Implements the UI components for the search interface
- **pagination.rs**: Manages pagination logic for search results
- **utils.rs**: Provides helper functions used throughout the application

### Dependencies

The project uses the following key dependencies:
- `dioxus`: For building the user interface (v0.7.1)
- `mediathekviewweb`: For accessing the Mediathekviewweb API
- `reqwest`: For making HTTP requests
- `serde`: For JSON serialization/deserialization
- `chrono`: For date and time handling

### Features

The application supports multiple platforms through Dioxus features:
- `web`: Web platform support
- `desktop`: Desktop application support (default)
- `mobile`: Mobile application support
```

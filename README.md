# 🚀 Orbit

A sleek, Siri-inspired overlay assistant built with Tauri + React + TypeScript. **This is a native desktop application** that runs cross-platform on macOS, Windows, and Linux.

## ✨ Features

- **Siri-style Interface**: Beautiful glassmorphism overlay with transparent background
- **Voice Input**: Click the microphone to use speech recognition
- **Text Input**: Type your queries directly
- **Always On Top**: Stays visible over other applications
- **Native Performance**: Built with Tauri for lightweight, fast desktop performance
- **Cross-platform**: Works on macOS, Windows, and Linux

## 🎯 Usage

### Launching the App
```bash
pnpm tauri dev
```

### Interacting with Orbit
- **Type**: Click the input field and type your question
- **Voice**: Click the microphone button (🎤) to speak
- **Hide**: Press `Escape` or click outside to hide the overlay
- **Submit**: Press `Enter` or click the input field to send your query

### Example Queries
- "Hello" - Get a friendly greeting
- "What's the weather like?" - Weather inquiry (placeholder)
- "What time is it?" - Time query
- Any other text - General interaction

## 🛠️ Development

### Prerequisites
- [Node.js](https://nodejs.org/) (v16 or higher)
- [pnpm](https://pnpm.io/) package manager
- [Rust](https://rustup.rs/) (for Tauri backend)

### Setup
```bash
# Clone the repository
git clone <your-repo-url>
cd orbit

# Install dependencies
pnpm install

# Run development server
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Project Structure
```
orbit/
├── src/                    # React frontend
│   ├── App.tsx            # Main application component
│   ├── App.css            # Siri-style glassmorphism styles
│   └── main.tsx           # React entry point
├── src-tauri/             # Tauri backend
│   ├── src/
│   │   └── main.rs        # Rust backend with processing
│   └── tauri.conf.json    # Tauri configuration
└── package.json           # Dependencies and scripts
```

## 🎨 Customization

### Styling
All styles are in `src/App.css` with CSS custom properties for easy theming:
- Glassmorphism effects with `backdrop-filter`
- Smooth animations and transitions
- Responsive design for different screen sizes

### Backend
The processing happens in `src-tauri/src/main.rs`:
- Replace the `process_query` function with your service integration
- Add API keys and configuration as needed
- Extend with more sophisticated capabilities

### Window Configuration
Modify `src-tauri/tauri.conf.json` to adjust:
- Window size and position
- Transparency and decorations
- Security settings

## 🔧 Features to Add

- [ ] Global hotkey support (Cmd+Space)
- [ ] Service integration (OpenAI, Anthropic, etc.)
- [ ] Conversation history
- [ ] Custom themes and appearance
- [ ] Plugin system for extensions
- [ ] Voice synthesis for responses
- [ ] Multiple model support

## 📝 License

MIT License - feel free to use this for your own projects!

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

---

**Built with ❤️ using Tauri, React, and TypeScript**

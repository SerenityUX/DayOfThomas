# Day of Thomas - Audio Journal with Contribution Graph

A terminal-based journaling application that records audio entries, transcribes them using OpenAI's Whisper API, and visualizes your journaling streak with a GitHub-style contribution graph.

<img width="647" alt="Contribution Graph Example" src="https://github.com/user-attachments/assets/5d0b593a-4ca8-4e83-b5c6-abbe47363f12" />

## Features

- Record audio journal entries directly from your terminal
- Automatic transcription using OpenAI's Whisper API
- Color-coded entries based on emotional content analysis
- GitHub-style contribution visualization
- Simple terminal interface

## Prerequisites

1. [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
2. OpenAI API key (for Whisper transcription and color analysis)
3. macOS (currently only tested on macOS)

## Installation

1. Clone the repository:
   ```bash
   git clone [your-repo-url]
   cd day_of_thomas
   ```

2. Set up your OpenAI API key:
   - Copy the example environment file:
     ```bash
     cp .env.example .env
     ```
   - Edit `.env` and replace `your-openai-api-key-here` with your actual OpenAI API key

3. Build the release version:
   ```bash
   # Make the build script executable
   chmod +x build.sh
   
   # Run the build script
   ./build.sh
   ```
   This will:
   - Compile the optimized release version
   - Create a `release` directory with all necessary files
   - Set up the required directories (audio, models)
   - Copy your OpenAI API key to the release directory
   - Create a launcher script for easy execution

## Usage

There are two ways to run the application:

### Method 1: Development Mode (for testing/development)
```bash
cargo run
```

### Method 2: Release Mode (for daily use)
After building the release version:
1. Navigate to the `release` directory in Finder
2. Double-click `launch.command`
3. The application will run with all optimizations enabled

You can also:
- Move the `release` directory anywhere on your computer
- Create an alias to `launch.command` in your Applications folder
- Pin the launcher to your dock for quick access

## How It Works

1. The application shows your journaling contribution graph
2. If you haven't made an entry today:
   - Press Enter to start recording
   - Speak your journal entry
   - Press Enter again to stop recording
3. Your entry will be:
   - Transcribed using OpenAI's Whisper API
   - Analyzed for emotional content to determine the color
   - Saved to your journal history
4. The contribution graph updates to show your new entry

## Troubleshooting

1. **"No such file or directory" error**
   - Make sure you're running the application from the correct directory
   - For release mode, always use `launch.command`
   - Don't move files out of the `release` directory individually

2. **Audio recording issues**
   - Check if your microphone is working and properly selected in system settings
   - Try speaking closer to the microphone
   - Ensure you have permission granted for microphone access

3. **API errors**
   - Verify your OpenAI API key is correctly set in the `.env` file
   - Check your internet connection
   - Ensure your OpenAI account has available credits

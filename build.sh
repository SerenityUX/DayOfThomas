#!/bin/bash

# Build the release version
cargo build --release

# Create necessary directories
mkdir -p ./release/audio
mkdir -p ./release/models

# Copy the executable
cp ./target/release/day_of_thomas ./release/

# Copy or create necessary files
cp .env.example ./release/.env.example
cp .env ./release/.env
touch ./release/analysis.json
echo '[]' > ./release/analysis.json

# Create launcher script
cat > ./release/launch.command << 'EOL'
#!/bin/bash

# Get the directory where the script is located
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Change to the directory
cd "$DIR"

# Run the program
./day_of_thomas
EOL

# Make the executable and launcher runnable
chmod +x ./release/day_of_thomas
chmod +x ./release/launch.command

echo "Build complete! The application is in the ./release directory."
echo "Double-click launch.command to run the application." 
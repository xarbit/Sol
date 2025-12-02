#!/bin/bash

echo "🧪 Running Sol Calendar Flatpak..."
echo

# Run with debug output
RUST_LOG=debug flatpak run io.github.xarbit.SolCalendar "$@"

# Alternative: Run with shell access for debugging
# Uncomment the following to open a shell inside the sandbox:
# echo "Opening shell in Flatpak sandbox..."
# flatpak run --command=bash io.github.xarbit.SolCalendar

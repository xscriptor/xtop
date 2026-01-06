#!/bin/bash

# xtop uninstaller

APP_NAME="xtop"
INSTALL_DIR="/usr/local/bin"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Uninstalling $APP_NAME...${NC}"

if [ -f "$INSTALL_DIR/$APP_NAME" ]; then
    if [ -w "$INSTALL_DIR/$APP_NAME" ]; then
        rm "$INSTALL_DIR/$APP_NAME"
    else
        echo "Sudo permissions required to remove $APP_NAME"
        sudo rm "$INSTALL_DIR/$APP_NAME"
    fi
    echo -e "${GREEN}$APP_NAME removed successfully.${NC}"
else
    echo -e "${RED}$APP_NAME not found in $INSTALL_DIR.${NC}"
    exit 1
fi

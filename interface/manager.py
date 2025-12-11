# By: Oliver Osmond
# Date:2025-11-30
# Program Details: Generated manager file for the PyQT framework.

import sys, os
import interface.page_1
import interface.page_2
import interface.manager_core as core

if getattr(sys, 'frozen', False):
    import PySide6
    os.environ["QT_QPA_PLATFORM_PLUGIN_PATH"] = os.path.join(
        sys._MEIPASS, "PySide6", "plugins", "platforms"
    )


# Create screen instances
screen1 = interface.page_1.MainWindow()
screen2 = interface.page_2.MainWindow()

# Capture screen names and add to widget (preserving original pattern)
screen_names = [name for name in globals() if name.startswith('screen')]
for name in screen_names:
    screen_widget = globals()[name]
    core.widget.addWidget(screen_widget)

# Configure widget
core.widget.resize(screen1.size())
core.widget.setWindowTitle(screen1.windowTitle())

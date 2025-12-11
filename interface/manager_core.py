# By: Oliver Osmond
# Date:2025-11-30
# Program Details: Core manager file for the PyQT framework.

import sys, os, contextlib
from PySide6.QtWidgets import (QStackedWidget, QApplication)

if getattr(sys, 'frozen', False):
    sys.path.insert(0, os.path.join(sys._MEIPASS, "gui"))
    import PySide6
    os.environ["QT_QPA_PLATFORM_PLUGIN_PATH"] = os.path.join(
        sys._MEIPASS, "PySide6", "plugins", "platforms"
    )

@contextlib.contextmanager
def image_gui_path():
    if getattr(sys, 'frozen', False):
        gui_path = os.path.join(sys._MEIPASS)
    else:
        gui_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "gui")
    try:
        os.chdir(gui_path)
        yield
    finally:
        os.chdir(os.path.dirname(os.path.abspath(__file__)))

app = QApplication(sys.argv)
widget = QStackedWidget()

def start():
    widget.show()
    sys.exit(app.exec())
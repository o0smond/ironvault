# By: Oliver Osmond
# Date: 2025-11-30
# Program Details: Python logic for starting page. Manages rust flow.

import os, sys
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import manager_core as core
from PySide6.QtGui import QPixmap
from PySide6.QtWidgets import QMainWindow
from page_1_ui import Ui_MainWindow
import vault_core

if getattr(sys, 'frozen', False):
    import PySide6
    os.environ["QT_QPA_PLATFORM_PLUGIN_PATH"] = os.path.join(
        sys._MEIPASS, "PySide6", "plugins", "platforms"
    )


if __name__ == "__main__":    
    core.start()
    
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
storage_path = os.path.join(BASE_DIR, "..", "vault_core", "src", "storage.json")
storage_path = os.path.realpath(storage_path)

class MainWindow(QMainWindow, Ui_MainWindow):
    def __init__(self, parent=None):
        super().__init__(parent)
        with core.image_gui_path():
            self.setupUi(self)
            self.adjustSize()
            self.setFixedSize(self.size())
    
    def btn_ok_a(self):
        password = self.txt_input.text()
        if password == "":
            self.lbl_welcome.setText("Please enter a password.")
        else:
            rmsg = vault_core.pg1_startup(password, storage_path)
            if rmsg == "ok":
                core.widget.setCurrentWidget(core.screen2)
                core.widget.resize(862, 611)
                vault_core.unlock_vault()
                core.screen2.setup()
            else:
                self.lbl_welcome.setText(rmsg)
        
        

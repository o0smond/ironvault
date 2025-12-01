
# By: <Your Name Here>
# Date: 2025-11-30
# Program Details: <Program Description Here>

import os, sys
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import manager
from PySide6.QtGui import QPixmap
from PySide6.QtWidgets import QMainWindow
from gui.page_1_ui import Ui_MainWindow
import vault_core

if __name__ == "__main__":    
    manager.start()

class MainWindow(QMainWindow, Ui_MainWindow):
    def __init__(self, parent=None):
        super().__init__(parent)
        with manager.image_gui_path():
            self.setupUi(self)
    
    def btn_ok_a(self):
        password = self.txt_input.text()
        if password == "":
            self.lbl_welcome.setText("Please enter a password.")
        else:
            rmsg = vault_core.pg1_startup(password)
            if rmsg == "ok":
                manager.widget.setCurrentWidget(manager.screen2)
            else:
                self.lbl_welcome.setText(rmsg)
        
        

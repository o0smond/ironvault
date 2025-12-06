# By: <Your Name Here>
# Date: 2025-11-30
# Program Details: <Program Description Here>

import os, sys
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import manager
from PySide6.QtWidgets import QInputDialog
from PySide6.QtGui import QPixmap
from PySide6.QtWidgets import QMainWindow
from gui.page_2_ui import Ui_MainWindow
import vault_core

if __name__ == "__main__":    
    manager.start()

class MainWindow(QMainWindow, Ui_MainWindow):
    def __init__(self, parent=None):
        super().__init__(parent)
        with manager.image_gui_path():
            self.setupUi(self)
            
    def setup():
        pass
            
    def btn_add_a(self):
        while True:
            user = QInputDialog.getText(self, 'What is the username for the entry?', 'Username:')
            password = QInputDialog.getText(self, 'What is the password for the entry?', 'Password:')
            if user[0] == "":
                user = QInputDialog.getText(self, 'Username cannot be empty', 'Username:')
            elif password[0] == "":
                password = QInputDialog.getText(self, 'Password cannot be empty', 'Password:')
            else:
                break
        self.txt_main.setText(vault_core.add_password(user[0], password[0]))
        
        
        
    
    def btn_e_r_a(self):
        pass
        
    def btn_exit_a(self):
        pass
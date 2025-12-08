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
            
    def setup(self):
        self.txt_main.setText(self.map_cleaner(str(vault_core.print_map())))
    
    
    def map_cleaner(self, map):
        map = map.replace("{\n", "")
        map = map.replace("}", "")
        map = map.replace('"', "")
        map = map.replace(",", "\n")
        return map
                
            
    def btn_add_a(self):
        while True:
            user = QInputDialog.getText(self, 'What is the username for the entry?', 'Username:')
            if user[0] == "":
                pass
            else:
                break
        while True:
            password = QInputDialog.getText(self, 'What is the password for the entry?', 'Password:')
            if password[0] == "":
                pass
            else:
                break
        self.txt_main.setText(self.map_cleaner(str(vault_core.add_password(user[0], password[0])))) 
    
    def btn_e_r_a(self):
        option = QInputDialog.getItem(self, "Edit or Remove", "Edit or Remove", ["Edit", "Remove"])
        if option[0] == "Edit":
            pass
        elif option[0] == "Remove":
            while True:
                user = QInputDialog.getText(self, 'What is the username for the entry?', 'Username:')
                if user[0] == "":
                    pass
                else:
                    break
            self.txt_main.setText(self.map_cleaner(str(vault_core.delete(user[0]))))
        self.setup()
        
    def btn_exit_a(self):
        rmsg = vault_core.lock_vault()
        if rmsg == "ok":
            exit(0)
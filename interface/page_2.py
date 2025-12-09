# By: Oliver Osmond
# Date: 2025-11-30
# Program Details: Python logic for main page. Manages rust flow, verifies user inputs, translates rust outputs.

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
        user, ok = QInputDialog.getText(self, 'Add Entry', 'Username:')
        if not ok or not user.strip():
            return
            
        password, ok = QInputDialog.getText(self, 'Add Entry', 'Password:')
        if not ok or not password.strip():
            return
            
        result = vault_core.add_password(user, password)
        self.txt_main.setText(self.map_cleaner(str(result)))
    
    def btn_e_r_a(self):
        option, ok = QInputDialog.getItem(self, "Edit or Remove", "Choose action:", ["Edit", "Remove"], 0, False)
        if not ok:
            return
            
        user, ok = QInputDialog.getText(self, 'Select Entry', 'Username:')
        if not ok or not user.strip():
            return
            
        if option == "Edit":
            new_user, ok = QInputDialog.getText(self, 'Edit Entry', 'New username:')
            if not ok or not new_user.strip():
                return
                
            new_pass, ok = QInputDialog.getText(self, 'Edit Entry', 'New password:')
            if not ok or not new_pass.strip():
                return
                
            vault_core.delete(user)
            result = vault_core.add_password(new_user, new_pass)
            self.txt_main.setText(self.map_cleaner(str(result)))
            
        elif option == "Remove":
            result = vault_core.delete(user)
            self.txt_main.setText(self.map_cleaner(str(result)))
            
        self.setup()
        
    def btn_exit_a(self):
        rmsg = vault_core.lock_vault()
        if rmsg == "ok":
            exit(0)
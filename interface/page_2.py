# By: Oliver Osmond
# Date: 2025-11-30
# Program Details: Python logic for main page. Manages rust flow, verifies user inputs, translates rust outputs.

import os, sys
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import manager_core as core
from PySide6.QtWidgets import QInputDialog, QMessageBox
from PySide6.QtGui import QPixmap
from PySide6.QtWidgets import QMainWindow
from page_2_ui import Ui_MainWindow
import vault_core

if getattr(sys, 'frozen', False):
    import PySide6
    os.environ["QT_QPA_PLATFORM_PLUGIN_PATH"] = os.path.join(
        sys._MEIPASS, "PySide6", "plugins", "platforms"
    )


if __name__ == "__main__":    
    core.start()

class MainWindow(QMainWindow, Ui_MainWindow):
    def __init__(self, parent=None):
        super().__init__(parent)
        with core.image_gui_path():
            self.setupUi(self)
            self.adjustSize()
            self.setFixedSize(self.size())
            
    text_warning = '*Please note these illegal characters:\n", {, }, and , \nwill be automatically removed.\n'
            
    def setup(self):
        self.txt_main.setText(self.map_cleaner(str(vault_core.print_map())))
    
    
    def map_cleaner(self, map):
        map = map.replace("{\n", "")
        map = map.replace("{", "")
        map = map.replace("}", "")
        map = map.replace('"', "")
        map = map.replace(",", "\n")
        return map
    
    def input_cleaner(self, input):
        input = input.replace("{\n", "")
        input = input.replace("{", "")
        input = input.replace("}", "")
        input = input.replace('"', "")
        input = input.replace(",", "")
        return input
    
    def gen_userlist(self, clean_map):
        user_list = []
        for line in clean_map.split("\n"):
            if ": " in line:
                user, _ = line.split(": ", 1)
                user_list.append(user.strip())
        return user_list
    
    def btn_add_a(self):
        userlist = self.gen_userlist(self.map_cleaner(str(vault_core.print_map())))
        
        user, ok = QInputDialog.getText(self, 'Add Entry', f'{self.text_warning}Username:')
        user = self.input_cleaner(user)
        if not ok:
            return
        if not user.strip():
            QMessageBox.information(self, "Error", "Username cannot be blank")
            return
        for i in range(len(userlist)):
            if user == userlist[i]:
                QMessageBox.information(self, "Error", "Username already exists.")
                return
            
        password, ok = QInputDialog.getText(self, 'Add Entry', f'{self.text_warning}Password:')
        password = self.input_cleaner(password)
        if not ok:
            return
        if not user.strip():
            QMessageBox.information(self, "Error", "Password cannot be blank")
            return
            
        result = vault_core.add_password(user, password)
        self.txt_main.setText(self.map_cleaner(str(result)))
    
    def btn_e_r_a(self):
        option, ok = QInputDialog.getItem(self, "Edit or Remove", "Choose action:", ["Edit", "Remove"], 0, False)
        if not ok:
            return
        
        userlist = self.gen_userlist(self.map_cleaner(str(vault_core.print_map())))
            
        user, ok = QInputDialog.getText(self, 'Select Entry', f'{self.text_warning}Username:')
        user = self.input_cleaner(user)
        if not ok:
            return
        if not user.strip():
            QMessageBox.information(self, "Error", "Username cannot be blank")
            return
        
        if option == "Edit":
            new_user, ok = QInputDialog.getText(self, 'Edit Entry', f'{self.text_warning}New username:')
            new_user = self.input_cleaner(new_user)
            if not ok:
                return
            if not user.strip():
                QMessageBox.information(self, "Error", "New username cannot be blank")
                return
            for i in range(len(userlist)):
                if new_user == userlist[i]:
                    QMessageBox.information(self, "Error", "Username already exists.")
                    return
            new_pass, ok = QInputDialog.getText(self, 'Edit Entry', f'{self.text_warning}New password:')
            new_pass = self.input_cleaner(new_pass)
            if not ok:
                return
            if not user.strip():
                QMessageBox.information(self, "Error", "New password cannot be blank")
                return
                
            vault_core.delete(user)
            result = vault_core.add_password(new_user, new_pass)
            self.txt_main.setText(self.map_cleaner(str(result)))
            
        elif option == "Remove":
            result = vault_core.delete(user)
            self.txt_main.setText(self.map_cleaner(str(result)))
            
        self.setup()
        
    def btn_save_a(self):
        rmsg = vault_core.lock_vault()
        if rmsg == "ok":
            QMessageBox.information(self, "Success", "Passwords saved!")
        else:
            QMessageBox.information(self, "Error", "Passwords failed to save.")
        
    def btn_exit_a(self):
        rmsg = vault_core.lock_vault()
        if rmsg == "ok":
            exit(0)
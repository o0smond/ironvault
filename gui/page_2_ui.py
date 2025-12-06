# -*- coding: utf-8 -*-

################################################################################
## Form generated from reading UI file 'page_2.ui'
##
## Created by: Qt User Interface Compiler version 6.9.1
##
## WARNING! All changes made in this file will be lost when recompiling UI file!
################################################################################

from PySide6.QtCore import (QCoreApplication, QDate, QDateTime, QLocale,
    QMetaObject, QObject, QPoint, QRect,
    QSize, QTime, QUrl, Qt)
from PySide6.QtGui import (QBrush, QColor, QConicalGradient, QCursor,
    QFont, QFontDatabase, QGradient, QIcon,
    QImage, QKeySequence, QLinearGradient, QPainter,
    QPalette, QPixmap, QRadialGradient, QTransform)
from PySide6.QtWidgets import (QApplication, QHBoxLayout, QMainWindow, QPushButton,
    QSizePolicy, QSpacerItem, QTextBrowser, QVBoxLayout,
    QWidget)

class Ui_MainWindow(object):
    def setupUi(self, MainWindow):
        if not MainWindow.objectName():
            MainWindow.setObjectName(u"MainWindow")
        MainWindow.resize(862, 611)
        self.centralwidget = QWidget(MainWindow)
        self.centralwidget.setObjectName(u"centralwidget")
        self.centralwidget.setStyleSheet(u"background-color: rgb(255, 190, 111);")
        self.horizontalLayout = QHBoxLayout(self.centralwidget)
        self.horizontalLayout.setObjectName(u"horizontalLayout")
        self.verticalLayout = QVBoxLayout()
        self.verticalLayout.setObjectName(u"verticalLayout")
        self.btn_add = QPushButton(self.centralwidget)
        self.btn_add.setObjectName(u"btn_add")

        self.verticalLayout.addWidget(self.btn_add)

        self.btn_e_r = QPushButton(self.centralwidget)
        self.btn_e_r.setObjectName(u"btn_e_r")

        self.verticalLayout.addWidget(self.btn_e_r)

        self.btn_exit = QPushButton(self.centralwidget)
        self.btn_exit.setObjectName(u"btn_exit")

        self.verticalLayout.addWidget(self.btn_exit)

        self.verticalSpacer = QSpacerItem(20, 40, QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Expanding)

        self.verticalLayout.addItem(self.verticalSpacer)


        self.horizontalLayout.addLayout(self.verticalLayout)

        self.txt_main = QTextBrowser(self.centralwidget)
        self.txt_main.setObjectName(u"txt_main")
        self.txt_main.setStyleSheet(u"background-color: rgb(255, 255, 255);")

        self.horizontalLayout.addWidget(self.txt_main)

        MainWindow.setCentralWidget(self.centralwidget)

        self.retranslateUi(MainWindow)
        self.btn_add.clicked.connect(MainWindow.btn_add_a)
        self.btn_e_r.clicked.connect(MainWindow.btn_e_r_a)
        self.btn_exit.clicked.connect(MainWindow.btn_exit_a)

        QMetaObject.connectSlotsByName(MainWindow)
    # setupUi

    def retranslateUi(self, MainWindow):
        MainWindow.setWindowTitle(QCoreApplication.translate("MainWindow", u"Page 1", None))
        self.btn_add.setText(QCoreApplication.translate("MainWindow", u"Add Password", None))
        self.btn_e_r.setText(QCoreApplication.translate("MainWindow", u"Remove/Edit \n"
"password", None))
        self.btn_exit.setText(QCoreApplication.translate("MainWindow", u"Exit + Save", None))
    # retranslateUi


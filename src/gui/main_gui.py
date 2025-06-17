import sys
from PySide6 import QtCore
from PySide6.QtWidgets import (
    QApplication, QWidget, QMainWindow, QHBoxLayout, QVBoxLayout, QLabel, QStackedWidget, QGraphicsDropShadowEffect, QPushButton
)
from PySide6.QtGui import (
    QColor, 
)

import qtawesome as qta

from nav_bar import NavBar

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("SPLOT")

        styleFile = QtCore.QFile('./src/gui/stylesheet/main-gui.qss')
        styleFile.open(QtCore.QFile.OpenModeFlag.ReadOnly)
        self.setStyleSheet(styleFile.readAll().data().decode())

        self.setWindowFlags(QtCore.Qt.WindowType.FramelessWindowHint)

        self.plants = []

        self._setup_ui()

    def _setup_ui(self):
        self.central_widget = QWidget()
        self.central_widget.setObjectName('centralWidget')
        self.setCentralWidget(self.central_widget)
        self.main_layout = QVBoxLayout(self.central_widget)
        self.main_layout.setContentsMargins(0, 0, 0, 0)

        self.title_container = QWidget()
        self.title_container.setObjectName('titleContainer')
        self.title_layout = QHBoxLayout(self.title_container)
        self.title_layout.setContentsMargins(0, 0, 0, 0)
        # self.title_layout.setAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)
        self.main_layout.addWidget(self.title_container)

        self.title_layout.addStretch()
        
        self.app_title = QLabel("Senamby")
        self.app_title.setAlignment(QtCore.Qt.AlignmentFlag.AlignBottom)
        self.app_title.setObjectName("titleLabel")
        self.title_layout.addWidget(self.app_title)

        self.title_layout.addStretch()

        self.close_icon = qta.icon('mdi6.close', color=QColor(255, 255, 255))
        self.close_icon_ver = qta.icon('mdi6.close', color=QColor(200, 200, 200))
        
        self.close_button = QPushButton()
        self.close_button.setIcon(self.close_icon)
        
        self.close_button.setIconSize(self.close_button.size())
        self.close_button.setIconSize(QtCore.QSize(32, 32))
        self.close_button.setObjectName('closeButton')

        self.close_button.enterEvent = lambda event: self.close_button.setIcon(self.close_icon_ver)
        self.close_button.leaveEvent = lambda event: self.close_button.setIcon(self.close_icon)
        self.close_button.clicked.connect(self._handle_close_dialog)

        self.title_layout.addWidget(self.close_button)

        self.nav_toolbar = NavBar(self)
        self.main_layout.addWidget(self.nav_toolbar)

        self.main_content_widget = QStackedWidget()
        self.main_content_widget.setObjectName("mainContentWidget")
        self.main_layout.addWidget(self.main_content_widget)
        shadow_effect = QGraphicsDropShadowEffect(self)
        shadow_effect.setBlurRadius(10)
        shadow_effect.setColor(QColor(0, 0, 0, 40))
        shadow_effect.setOffset(0, 3)
        self.main_content_widget.setGraphicsEffect(shadow_effect)

    def _handle_close_dialog(self):
        self.close()

if __name__ == '__main__':
    app = QApplication()
    main_window = MainWindow()
    main_window.showMaximized()
    sys.exit(app.exec())
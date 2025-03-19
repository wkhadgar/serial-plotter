from collections.abc import Callable
from functools import partial
import sys

from PyQt5 import QtCore
from PyQt5.QtWidgets import QTabWidget, QApplication, QMainWindow

from .plotter_gui import PlotterGUI
from .analyzer_gui import AnalyzerGUI
        
class MainGUI(QMainWindow):
    def __init__(self, app_manager):
        super().__init__()
        self.app_manager = app_manager
        self.setWindowTitle("Control System GUI")
        self.setGeometry(100, 100, 1200, 800)

        self.tabs = QTabWidget()
        self.setCentralWidget(self.tabs)

        self.plotter_gui = PlotterGUI(self.app_manager)
        self.analyzer_gui = AnalyzerGUI(self.app_manager)

        self.tabs.addTab(self.plotter_gui, "PLOTTER")
        self.tabs.addTab(self.analyzer_gui, "ANALYZER")
        
        self.fullscreen_mode = False
        
        self.keyPressEvent = partial(self.key_press_handle, self.keyPressEvent)

    @staticmethod
    def start_gui(app_manager):
        app = QApplication(sys.argv)
        window = MainGUI(app_manager)
        window.showFullScreen()
        sys.exit(app.exec_())
        
    def key_press_handle(self, super_press_handler, ev):
        if ev.key() == QtCore.Qt.Key_Escape:
            sys.exit(0)
        if ev.key() == QtCore.Qt.Key_F:
            self.toggle_fullscreen()
        
    def toggle_fullscreen(self):
        if self.fullscreen_mode:
            self.plotter_gui.sidebar.show()
            self.main_layout.insertWidget(0, self.sidebar, 1)
            self.main_layout.setStretchFactor(self.sidebar, 1)
            self.main_layout.setStretchFactor(self.content_area, 4)
        else:
            self.plotter_gui.sidebar.hide()
            self.main_layout.setStretchFactor(self.content_area, 5)

        self.fullscreen_mode = not self.fullscreen_mode
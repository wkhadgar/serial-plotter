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
        
        self.hide_mode = False
        
        self.setFocusPolicy(QtCore.Qt.StrongFocus)
        self.installEventFilter(self) 

    def eventFilter(self, obj, event):
        if event.type() == QtCore.QEvent.KeyPress:
            print(f"Evento de tecla detectado: {event.key()}")
            if event.key() == QtCore.Qt.Key_F:
                self.toggle_hide_mode()
            elif event.key() == QtCore.Qt.Key_Escape:
                sys.exit(0)
            return True
        return super().eventFilter(obj, event)

    @staticmethod
    def start_gui(app_manager):
        app = QApplication(sys.argv)
        window = MainGUI(app_manager)
        window.showFullScreen()
        sys.exit(app.exec_())

    def key_press_handle(self, super_press_handler, ev):
        print(f"teste {ev.key()} {QtCore.Qt.Key_F}")
        if ev.key() == QtCore.Qt.Key_Escape:
            sys.exit(0)
        elif ev.key() == QtCore.Qt.Key_F or ev.key() == 16777216:
            self.toggle_hide_mode()

    def toggle_hide_mode(self):
        if self.hide_mode:
            if self.tabs.currentIndex() == 0:
                self.plotter_gui.sidebar.show()
                self.plotter_gui.layout.insertWidget(0, self.plotter_gui.sidebar, 1)
                self.plotter_gui.layout.setStretchFactor(self.plotter_gui.sidebar, 1)
                self.plotter_gui.layout.setStretchFactor(self.plotter_gui.plotter_gui, 4)
            elif self.tabs.currentIndex() == 1:
                self.analyzer_gui.sidebar.show()
                self.analyzer_gui.layout.insertWidget(0, self.analyzer_gui.sidebar, 1)
                self.analyzer_gui.layout.setStretchFactor(self.analyzer_gui.sidebar, 1)
                self.analyzer_gui.layout.setStretchFactor(self.analyzer_gui.plotter_gui, 4)
        else:
            if self.tabs.currentIndex() == 0:
                self.plotter_gui.sidebar.hide()
                self.analyzer_gui.layout.setStretchFactor(self.plotter_gui, 5)
            elif self.tabs.currentIndex() == 1:
                self.analyzer_gui.sidebar.hide()
                self.analyzer_gui.layout.setStretchFactor(self.analyzer_gui, 5)
        self.hide_mode = not self.hide_mode
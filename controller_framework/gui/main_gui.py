import sys

from PySide6 import QtCore
from PySide6.QtWidgets import QTabWidget, QApplication, QMainWindow

from .plotter_gui import PlotterGUI
from .analyzer_gui import AnalyzerGUI

class MainGUI(QMainWindow):
    def __init__(self, app_mirror):
        super().__init__()

        from controller_framework.core import AppManager
        assert isinstance(app_mirror, AppManager)
        self.app_mirror = app_mirror

        self.setWindowTitle("Control System GUI")
        self.setGeometry(100, 100, 1200, 800)

        self.tabs = QTabWidget()
        self.setCentralWidget(self.tabs)

        self.plotter_gui = PlotterGUI(app_mirror=self.app_mirror)
        self.analyzer_gui = AnalyzerGUI(app_mirror=self.app_mirror)

        self.tabs.addTab(self.plotter_gui, "PLOTTER")
        self.tabs.addTab(self.analyzer_gui, "ANALYZER")

        self.hide_mode = False

        self.setFocusPolicy(QtCore.Qt.FocusPolicy.StrongFocus)
        self.installEventFilter(self)

        self.plotter_gui.command_triggered.connect(self.send_command)

    def eventFilter(self, obj, event):
        if event.type() == QtCore.QEvent.Type.KeyPress:
            if event.key() == QtCore.Qt.Key.Key_F:
                self.toggle_hide_mode()
            elif event.key() == QtCore.Qt.Key.Key_Escape:
                sys.exit(0)
            return True
        return super().eventFilter(obj, event)

    @staticmethod
    def start_gui(app_mirror):
        app = QApplication(sys.argv)
        window = MainGUI(app_mirror)
        window.showFullScreen()
        print('[GUI] started')
        sys.exit(app.exec())

    def key_press_handle(self, super_press_handler, ev):
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
                self.plotter_gui.layout.setStretchFactor(
                    self.plotter_gui.plotter_gui, 4
                )
            elif self.tabs.currentIndex() == 1:
                self.analyzer_gui.sidebar.show()
                self.analyzer_gui.layout.insertWidget(0, self.analyzer_gui.sidebar, 1)
                self.analyzer_gui.layout.setStretchFactor(self.analyzer_gui.sidebar, 1)
                self.analyzer_gui.layout.setStretchFactor(
                    self.analyzer_gui.plotter_gui, 4
                )
        else:
            if self.tabs.currentIndex() == 0:
                self.plotter_gui.sidebar.hide()
                self.analyzer_gui.layout.setStretchFactor(self.plotter_gui, 5)
            elif self.tabs.currentIndex() == 1:
                self.analyzer_gui.sidebar.hide()
                self.analyzer_gui.layout.setStretchFactor(self.analyzer_gui, 5)
        self.hide_mode = not self.hide_mode

    @QtCore.Slot(str, object)
    def send_command(self, command, value):
        data = {
            "type": command,
            "payload": value
        }
        
        self.app_mirror.queue_from_gui.put(data)
        print(f"[MainGUI] Enviou '{command}' com valor {value} para o [APP]")
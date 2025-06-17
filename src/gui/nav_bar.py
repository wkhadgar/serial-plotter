import os
import qtawesome as qta
from PySide6 import QtCore
from PySide6.QtWidgets import (
    QHBoxLayout, QVBoxLayout, QWidget, QToolBar, QButtonGroup, QLabel, QToolButton, QGraphicsDropShadowEffect,
    QComboBox, QPushButton, QStackedWidget, QLineEdit, QRadioButton,
)
from PySide6.QtGui import (
    QAction, QColor
)

from utils_gui import CustomDialog, CustomCodeDialog

class addPlantDialog(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)

        self.dialog: CustomDialog = None

        self._setup_ui()

    def exec(self):
        if self.dialog is not None:
            self.dialog.exec()

    def _setup_ui(self):
        self.dialog_content_container = QWidget()
        self.dialog_content_layout = QVBoxLayout(self.dialog_content_container)

        current_dir = os.path.dirname(os.path.abspath(__file__))
        style_file_path = os.path.join(current_dir, 'stylesheet', 'nav-bar.qss')
        self.styleFile = QtCore.QFile(style_file_path)
        self.styleFile.open(QtCore.QFile.OpenModeFlag.ReadOnly)
        self.dialog_content_container.setStyleSheet(self.styleFile.readAll().data().decode())

        self.dialog = CustomDialog('Gerenciar plantas')

        self._create_modes_buttons()
        self._create_main_content()
        self._create_rodape_content()

        self.dialog.addWidget(self.dialog_content_container, stretch=1)

    def _create_rodape_content(self):
        self.rodape_container = QWidget()
        self.rodape_container.setObjectName('rodapeContainer')
        self.rodape_layout = QHBoxLayout(self.rodape_container)
        self.rodape_layout.setAlignment(QtCore.Qt.AlignmentFlag.AlignHCenter)
        self.ok_btn = QPushButton('Adicionar')
        self.rodape_layout.addWidget(self.ok_btn)
        self.dialog_content_layout.addWidget(self.rodape_container, stretch=0)

    def _create_main_content(self):
        self.content_container = QWidget()
        self.content_container.setObjectName('contentContainer')
        self.content_layout = QVBoxLayout(self.content_container)
        self.dialog_content_layout.addWidget(self.content_container, stretch=1)

        self.stacked_content_widget = QStackedWidget(self.content_container)
        self.content_layout.addWidget(self.stacked_content_widget)

        self._create_new_plant_page()
        self._create_existent_page()

    def _create_new_plant_page(self):
        self.new_plant_page = QWidget()
        self.new_plant_layout = QVBoxLayout(self.new_plant_page)
        self.new_plant_layout.addStretch(1)
        self.stacked_content_widget.addWidget(self.new_plant_page)

        self.plant_name_label = QLabel('Nome da planta:')
        self.plant_name_input = QLineEdit()
        self.new_plant_layout.addWidget(self.plant_name_label)
        self.new_plant_layout.addWidget(self.plant_name_input)

        self.plant_driver_label = QLabel('Tipo de driver:')
        self.new_plant_layout.addWidget(self.plant_driver_label)

        self.radio_driver_container = QWidget()
        self.radio_driver_layout = QHBoxLayout(self.radio_driver_container)
        self.new_plant_layout.addWidget(self.radio_driver_container)

        self.radio_driver_group = QButtonGroup(self.radio_driver_container)

        self.new_plant_default_driver = QRadioButton('Padrão')
        self.radio_driver_layout.addWidget(self.new_plant_default_driver)
        self.radio_driver_group.addButton(self.new_plant_default_driver, 0)
        self.new_plant_default_driver.setChecked(True)

        self.new_plant_custom_driver = QRadioButton('Personalizado')
        self.radio_driver_layout.addWidget(self.new_plant_custom_driver)
        self.radio_driver_group.addButton(self.new_plant_custom_driver, 1)

        self.radio_driver_group.idToggled.connect(self._handle_change_radio_driver)

        self.driver_content_container = QWidget()
        self.driver_content_container.setObjectName('driverContentContainer')
        self.driver_content_layout = QVBoxLayout(self.driver_content_container)

        self.stacked_driver_content = QStackedWidget(self.driver_content_container)
        self.driver_content_layout.addWidget(self.stacked_driver_content)

        self.new_plant_layout.addWidget(self.driver_content_container)

        self.default_driver_page = QWidget()
        self.default_driver_page_layout = QVBoxLayout(self.default_driver_page)

        self.default_driver_combo_label = QLabel("Selecione o Driver Padrão:")
        self.default_driver_page_layout.addWidget(self.default_driver_combo_label)

        self.default_driver_combo = QComboBox()
        self.default_driver_combo.addItems(['STM32', 'Arduino', 'RDATA'])
        self.default_driver_page_layout.addWidget(self.default_driver_combo)
        self.default_driver_page_layout.addStretch(1)
        self.stacked_driver_content.addWidget(self.default_driver_page)

        self._create_custom_driver_page()

    def _create_custom_driver_page(self):
        self.custom_driver_page = QWidget()
        self.custom_driver_layout = QVBoxLayout(self.custom_driver_page)
        
        self.stacked_driver_content.addWidget(self.custom_driver_page)

        self.add_code_btn = QPushButton('Abrir editor')
        self.custom_driver_layout.addWidget(self.add_code_btn)

        self.add_code_btn.clicked.connect(self._create_code_dialog)

    def _create_code_dialog(self):
        self.code_dialog = CustomCodeDialog('Editor de código')
        self.code_dialog.exec()

    def _change_custom_driver_option_page(self, button_id: int, checked: bool):
        if checked:
            self.stacked_driver_content.setCurrentIndex(button_id)
            self.adjustSize()

    def _create_existent_page(self):
        self.existent_plant_page = QWidget()
        self.existent_plant_layout = QVBoxLayout(self.existent_plant_page)
        self.existent_plant_layout.addStretch(1)
        self.stacked_content_widget.addWidget(self.existent_plant_page)

        self.radio_import_container = QWidget()
        self.radio_import_layout = QHBoxLayout(self.radio_import_container)
        self.existent_plant_layout.addWidget(self.radio_import_container)

        self.new_plant_default_driver = QRadioButton('teste')
        self.existent_plant_layout.addWidget(self.new_plant_default_driver)

        self.new_plant_custom_driver = QRadioButton('teste2')
        self.existent_plant_layout.addWidget(self.new_plant_custom_driver)

    def _create_modes_buttons(self):
        self.buttons_mode_container = QWidget()
        self.buttons_mode_container.setObjectName('btnsModeContainer')
        self.buttons_mode_layout = QHBoxLayout(self.buttons_mode_container)
        self.buttons_mode_group = QButtonGroup()
        self.buttons_mode_group.setExclusive(True)
        self.dialog_content_layout.addWidget(self.buttons_mode_container, stretch=0)

        self.new_plant_btn = QPushButton('Nova planta')
        self.new_plant_btn.setCheckable(True)
        self.buttons_mode_group.addButton(self.new_plant_btn, id=0)
        self.buttons_mode_layout.addWidget(self.new_plant_btn)

        self.existent_plant_btn = QPushButton('Planta existente')
        self.existent_plant_btn.setCheckable(True)
        self.buttons_mode_group.addButton(self.existent_plant_btn, id=1)
        self.buttons_mode_layout.addWidget(self.existent_plant_btn)

        self.new_plant_btn.setChecked(True) 

        self.buttons_mode_group.idToggled.connect(self._handle_change_content_page)
    
    def _handle_change_content_page(self, btn_id: int, checked: bool):
        if not checked:
            return
        
        self.stacked_content_widget.setCurrentIndex(btn_id)
        self.adjustSize()

    def _handle_change_radio_driver(self, btn_id: int, checked: bool):
        if not checked:
            return
        
        self.stacked_driver_content.setCurrentIndex(btn_id)
        self.adjustSize()

class NavBar(QWidget):
    def __init__(self, main_window):
        super().__init__()
        self.main_window = main_window
        current_dir = os.path.dirname(os.path.abspath(__file__))
        style_file_path = os.path.join(current_dir, 'stylesheet', 'nav-bar.qss')
        self.styleFile = QtCore.QFile(style_file_path)
        self.styleFile.open(QtCore.QFile.OpenModeFlag.ReadOnly)

        self.nav_layout = QHBoxLayout(self)
        self._setup_ui()

    def _setup_ui(self):
        self.nav_toolbar = QToolBar("Navegação da Planta")
        self.nav_toolbar.setMovable(False)
        self.nav_toolbar.setFloatable(False)
        self.nav_toolbar.setIconSize(QtCore.QSize(32, 32))
        self.nav_toolbar.addWidget(QLabel("   "))
        self.nav_toolbar.setStyleSheet(self.styleFile.readAll().data().decode())

        self.nav_layout.addWidget(self.nav_toolbar)

        self.plant_buttons_group = QButtonGroup(self)
        self.plant_buttons_group.setExclusive(True)

        add_icon = qta.icon('ri.add-line', color=QColor('#ffffff'))
        self.add_plant_action = QAction(add_icon, '', self)
        self.add_plant_action.triggered.connect(self._add_plant_window)
        self.nav_toolbar.addAction(self.add_plant_action) 
        add_plant_button = self.nav_toolbar.widgetForAction(self.add_plant_action)
        add_plant_button.setObjectName("addPlantButton")

        save_icon = qta.icon('ri.save-line', color=QColor('#ffffff'))
        self.save_plant_action = QAction(save_icon, "", self)
        self.nav_toolbar.addAction(self.save_plant_action)
        save_plant_button = self.nav_toolbar.widgetForAction(self.save_plant_action)
        save_plant_button.setObjectName("savePlantButton")

        self._create_test_plant_button("Planta 1")
        self._create_test_plant_button("Planta 2")
        self._create_test_plant_button("Planta 3")
        self._create_test_plant_button("Planta 4")

        for tool_button in self.nav_toolbar.findChildren(QToolButton):
            shadow_effect = QGraphicsDropShadowEffect(self)
            shadow_effect.setBlurRadius(10)
            shadow_effect.setColor(QColor(0, 0, 0, 40))
            shadow_effect.setOffset(3, 3)
            tool_button.setGraphicsEffect(shadow_effect)

    def _add_plant_window(self):
        dialog = addPlantDialog(self.main_window)
        # dialog.plant_added.connect(self._add_new_plant_from_dialog)
        # dialog.plants_loaded.connect(self._handle_loaded_plants)
        dialog.exec()

    def _create_test_plant_button(self, txt):
        test_button = QToolButton()
        test_button.setText(txt)
        test_button.setCheckable(True)
        test_button.setToolButtonStyle(QtCore.Qt.ToolButtonTextOnly)

        self.plant_buttons_group.addButton(test_button)
        self.nav_toolbar.insertWidget(None, test_button)

        return test_button
import os
import qtawesome as qta
from PySide6 import QtCore
from PySide6.QtWidgets import (
    QHBoxLayout, QVBoxLayout, QApplication, QWidget, QTextEdit, QLabel, QDialog, QPushButton, QGraphicsDropShadowEffect,
    QFileDialog, 
)
from PySide6.QtGui import (
    QMouseEvent, QColor, QSyntaxHighlighter, QTextCharFormat, QFont, QFontMetrics,
)

class CustomDialog(QDialog):
    def __init__(self, title=''):
        super().__init__()
        self.win_title = title

        self._old_pos = None

        self.setWindowFlags(QtCore.Qt.WindowType.FramelessWindowHint)
        self.setAttribute(QtCore.Qt.WidgetAttribute.WA_TranslucentBackground)

        self.dialog_layout = QVBoxLayout(self)

        self.setMinimumSize(400, 400)
        self.setMaximumSize(900, 800)

        self.main_container = QWidget()
        self.main_container.setObjectName('mainContainer')
        self.main_layout = QVBoxLayout(self.main_container)
        self.main_layout.setContentsMargins(0, 0, 0, 0)
        self.main_layout.setSpacing(0)

        shadow_effect = QGraphicsDropShadowEffect(self)
        shadow_effect.setBlurRadius(10)
        shadow_effect.setColor(QColor(0, 0, 0, 40))
        shadow_effect.setOffset(0, 3)
        self.main_container.setGraphicsEffect(shadow_effect)

        self.dialog_layout.addWidget(self.main_container)

        self._create_dialog_title()
        self._set_qss('custom-dialog.qss')

        self.content_container = QWidget()
        self.content_container.setObjectName('contentDialogContainer')
        self.content_layout = QVBoxLayout(self.content_container)
        self.content_layout.setContentsMargins(0, 0, 0, 0)
        self.content_layout.setSpacing(0)
        self.main_layout.addWidget(self.content_container, stretch=1)

    def addWidget(self, widget: QWidget, stretch=None):
        self.content_layout.addWidget(widget, stretch=stretch)
    
    def showEvent(self, event):
        super().showEvent(event)
        self.layout().activate()

        size = self.layout().minimumSize()
        size = size.expandedTo(self.minimumSize())
        size = size.boundedTo(self.maximumSize())
        self.resize(size)

        self._center_on_screen()

    def _center_on_screen(self):
        screen_geo = QApplication.primaryScreen().availableGeometry()
        geo = self.frameGeometry()
        geo.moveCenter(screen_geo.center())
        self.move(geo.topLeft())

    def _set_qss(self, file=''):
        if file == '':
            return

        current_dir = os.path.dirname(os.path.abspath(__file__))
        style_file_path = os.path.join(current_dir, 'stylesheet', file)
        self.styleFile = QtCore.QFile(style_file_path)
        self.styleFile.open(QtCore.QFile.OpenModeFlag.ReadOnly)
        self.setStyleSheet(self.styleFile.readAll().data().decode())

    def _create_dialog_title(self):
        self.title_container = QWidget()
        self.title_container.setObjectName('titleDialogContainer')

        self.title_layout = QHBoxLayout(self.title_container)
        self.title_layout.setAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)

        self.dialog_title = QLabel(self.win_title)
        self.title_layout.addWidget(self.dialog_title)
        self.dialog_title.setAlignment(QtCore.Qt.AlignmentFlag.AlignLeft)

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

        self.main_layout.addWidget(self.title_container)

    def mousePressEvent(self, event: QMouseEvent):
        if event.button() == QtCore.Qt.MouseButton.LeftButton and \
           self.title_container.underMouse():
            self._old_pos = event.globalPosition().toPoint()
        super().mousePressEvent(event)
    
    def mouseMoveEvent(self, event: QMouseEvent):
        if event.buttons() == QtCore.Qt.MouseButton.LeftButton and self._old_pos:
            delta = event.globalPosition().toPoint() - self._old_pos
            self.move(self.pos() + delta)
            self._old_pos = event.globalPosition().toPoint()
        super().mouseMoveEvent(event)

    def mouseReleaseEvent(self, event: QMouseEvent):
        self._old_pos = None
        super().mouseReleaseEvent(event)

    def _handle_close_dialog(self):
        self.close()

class CustomCodeDialog(QWidget):
    class CodeEdit(QTextEdit):
        class PythonHighlighter(QSyntaxHighlighter):
            def __init__(self, document):
                super().__init__(document)

                # --- Definição de formatos ---
                # Palavras-chave
                keyword_format = QTextCharFormat()
                keyword_format.setForeground(QColor("#569CD6"))
                keyword_format.setFontWeight(QFont.Bold)
                keywords = [
                    "and", "as", "assert", "break", "class", "continue", "def", "del",
                    "elif", "else", "except", "finally", "for", "from", "global",
                    "if", "import", "in", "is", "lambda", "nonlocal", "not", "or",
                    "pass", "raise", "return", "try", "while", "with", "yield"
                ]

                # Builtins
                builtin_format = QTextCharFormat()
                builtin_format.setForeground(QColor("#9CDCFE"))
                builtins = [
                    "True", "False", "None", "list", "dict", "set", "tuple", "int",
                    "float", "str", "print", "len", "open", "range", "isinstance"
                ]

                # Decorators
                decorator_format = QTextCharFormat()
                decorator_format.setForeground(QColor("#C586C0"))
                decorator_format.setFontItalic(True)

                # Números
                number_format = QTextCharFormat()
                number_format.setForeground(QColor("#B5CEA8"))

                # Comentários
                comment_format = QTextCharFormat()
                comment_format.setForeground(QColor("#6A9955"))
                comment_format.setFontItalic(True)

                # Strings
                string_format = QTextCharFormat()
                string_format.setForeground(QColor("#CE9178"))

                # Lista de regras regex -> formato
                self.rules = []
                for kw in keywords:
                    pattern = QtCore.QRegularExpression(rf"\b{kw}\b")
                    self.rules.append((pattern, keyword_format))
                for bi in builtins:
                    pattern = QtCore.QRegularExpression(rf"\b{bi}\b")
                    self.rules.append((pattern, builtin_format))

                # Decorators: linhas que começam com @
                self.rules.append((QtCore.QRegularExpression(r"@\w+"), decorator_format))

                # Números (inteiros e floats)
                self.rules.append((QtCore.QRegularExpression(r"\b\d+(\.\d+)?\b"), number_format))

                # Comentários de linha
                self.rules.append((QtCore.QRegularExpression(r"#.*"), comment_format))

                # Strings simples (entre aspas simples ou duplas)
                self.rules.append((QtCore.QRegularExpression(r"\".*?\""), string_format))
                self.rules.append((QtCore.QRegularExpression(r"'.*?'"), string_format))

                # Delimitadores para strings multilinha
                self.tri_single = QtCore.QRegularExpression("'''")
                self.tri_double = QtCore.QRegularExpression('"""')
                self.multi_single_format = QTextCharFormat(string_format)
                self.multi_double_format = QTextCharFormat(string_format)

            def highlightBlock(self, text):
                for regex, fmt in self.rules:
                    it = regex.globalMatch(text)
                    while it.hasNext():
                        match = it.next()
                        start = match.capturedStart()
                        length = match.capturedLength()
                        self.setFormat(start, length, fmt)
                self.setCurrentBlockState(0)

        def __init__(self):
            super().__init__()
            self.highlighter = self.PythonHighlighter(self)
            self.tab_width = 4 

            self.setTabChangesFocus(False)

            metrics = QFontMetrics(self.font()) 
            self.setTabStopDistance(self.tab_width * metrics.averageCharWidth())
        
        def keyPressEvent(self, event):
            if event.key() == QtCore.Qt.Key.Key_Tab:
                self.insertPlainText(" " * self.tab_width)
            elif event.key() == QtCore.Qt.Key.Key_Backtab:
                cursor = self.textCursor()
                pos = cursor.position()
                cursor.movePosition(cursor.MoveOperation.Left, cursor.MoveMode.KeepAnchor, self.tab_width)
                selected = cursor.selectedText()
                if selected == " " * self.tab_width:
                    cursor.removeSelectedText()
                else:
                    cursor.clearSelection()
                    cursor.setPosition(pos - 1)
                self.setTextCursor(cursor)
            else:
                super().keyPressEvent(event)

    def __init__(self, title):
        super().__init__()
        self.dialog = CustomDialog(title)

        self.main_container = QWidget()
        self.main_layout = QVBoxLayout(self.main_container)
        self.dialog.addWidget(self.main_container, stretch=1)

        self.content_container = QWidget()
        self.content_layout = QVBoxLayout(self.content_container)
        self.main_layout.addWidget(self.content_container)

        self.rodape_container = QWidget()
        self.rodape_layout = QHBoxLayout(self.rodape_container)
        self.main_layout.addWidget(self.rodape_container)

        self._create_text_area()
        self._create_rodape()
    
    def exec(self):
        self.dialog.exec()

    def _create_text_area(self):
        self.text_editor = self.CodeEdit()
        self.text_editor.setMinimumSize(900, 700)
        self.content_layout.addWidget(self.text_editor, stretch=1, alignment=QtCore.Qt.AlignmentFlag.AlignCenter)

        self._set_text_from_file('default-driver-content.py')

    def _create_rodape(self):
        self.rodape_layout.setAlignment(QtCore.Qt.AlignmentFlag.AlignLeft)
        add_code_icon = qta.icon('ri.add-line')
        self.add_code_btn = QPushButton(text='Adicionar', icon=add_code_icon)

        self.add_code_btn.setIconSize(QtCore.QSize(32, 32))
        self.add_code_btn.setObjectName('addCodeBtn')

        self.rodape_layout.addWidget(self.add_code_btn)

        import_icon = qta.icon('ph.upload-simple')
        self.import_code_btn = QPushButton(text='Importar Código', icon=import_icon)

        self.import_code_btn.setIconSize(QtCore.QSize(32, 32))
        self.import_code_btn.setObjectName('importCodeBtn')

        self.import_code_btn.clicked.connect(self._handle_import_code_btn)
        self.rodape_layout.addWidget(self.import_code_btn)

    def _set_text_from_file(self, file_name = '', file_path=None):
        if file_path is None:
            current_dir = os.path.dirname(os.path.abspath(__file__))
            file_path = os.path.join(current_dir, 'content', file_name)

        self.default_text = QtCore.QFile(file_path)
        self.default_text.open(QtCore.QFile.OpenModeFlag.ReadOnly)

        self.text_editor.setText(self.default_text.readAll().toStdString())

    def _handle_import_code_btn(self):
        file_path, _ = QFileDialog.getOpenFileName(
        parent=self,
        caption="Importar código Python",
        dir="",
        filter="Python files (*.py)"
        )
        
        if file_path:
            self._set_text_from_file(file_path)


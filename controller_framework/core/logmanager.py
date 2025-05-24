import logging

class ColorLevelFormatter(logging.Formatter):
    COLORS = {
        'DEBUG': "\033[37m",    # branco
        'INFO': "\033[32m",     # verde
        'WARNING': "\033[33m",  # amarelo
        'ERROR': "\033[31m",    # vermelho
        'CRITICAL': "\033[41m", # fundo vermelho
    }
    RESET = "\033[0m"
    BRIGHT_WHITE = "\033[97m"

    def format(self, record):
        orig_level = record.levelname
        orig_prefix = getattr(record, 'prefix', None)

        color = self.COLORS.get(orig_level, self.RESET)
        record.levelname = f"{color}{orig_level}{self.RESET}"

        component = getattr(record, 'component', None)
        method    = getattr(record, 'method', None)
        if component and method:
            p = f"[{component}:{method}]"
        elif component:
            p = f"[{component}]"
        else:
            p = ""
        record.prefix = (f"{self.BRIGHT_WHITE}{p}{self.RESET} " if p else "")

        formatted = super().format(record)

        record.levelname = orig_level
        if orig_prefix is not None:
            record.prefix = orig_prefix
        else:
            delattr(record, 'prefix')

        return formatted


class ContextLoggerAdapter(logging.LoggerAdapter):
    def process(self, msg, kwargs):
        call_extra = kwargs.get('extra', {})
        merged = {**self.extra, **call_extra}
        kwargs['extra'] = merged
        return msg, kwargs


class LogManager:
    def __init__(self, name=__name__, level=logging.INFO):
        self.logger = logging.getLogger(name)
        self.logger.setLevel(level)

        self.logger.propagate = False

        if not self.logger.handlers:
            ch = logging.StreamHandler()
            fmt = '%(asctime)s - %(levelname)s - %(prefix)s%(message)s'
            ch.setFormatter(ColorLevelFormatter(fmt))
            self.logger.addHandler(ch)

    def get_logger(self, component: str = None, method: str = None):
        extra = {}
        if component:
            extra['component'] = component
        if method:
            extra['method'] = method
        return ContextLoggerAdapter(self.logger, extra)
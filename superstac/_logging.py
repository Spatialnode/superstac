"""SuperSTAC logging module"""

import logging


PACKAGE_NAME = "superstac"
LOGGING_FORMAT = f"{PACKAGE_NAME}:%(asctime)s - %(module)s.%(funcName)s - %(levelname)s - %(message)s"

logger = logging.getLogger(PACKAGE_NAME)


# Set the default logging level to INFO.
logger.setLevel(logging.INFO)


console_handler = logging.StreamHandler()

console_formatter = logging.Formatter(LOGGING_FORMAT)
console_handler.setFormatter(console_formatter)

logger.addHandler(console_handler)


def add_file_logging(file_path=f"{PACKAGE_NAME}.log"):
    """
    Add file logging to the logger.

    Args:
        file_path (str): Path to the log file. Defaults to `superstac.log`.

    Example:
        from superstac import add_file_logging
        # Configure custom file log
        add_file_logging("my_log.log")
    """
    for h in logger.handlers:
        if (
            isinstance(h, logging.FileHandler)
            and getattr(h, "baseFilename", None) == file_path
        ):
            return
    file_handler = logging.FileHandler(file_path)
    file_formatter = logging.Formatter(LOGGING_FORMAT)
    file_handler.setFormatter(file_formatter)
    logger.addHandler(file_handler)


def configure_logging(level=logging.WARNING):
    """
    Configure the logging level for the package.

    Args:
        level (int): Logging level (e.g., logging.DEBUG, logging.INFO, logging.WARNING, logging.ERROR)

    Example:
        # Set logging level to INFO
        from superstac import configure_logging
        import logging
        # Configure logging to INFO level
        configure_logging(logging.INFO)
    """
    logger.setLevel(level)
    if not any(isinstance(h, logging.StreamHandler) for h in logger.handlers):
        console_handler = logging.StreamHandler()
        console_handler.setLevel(level)
        console_handler.setFormatter(logging.Formatter(LOGGING_FORMAT))
        logger.addHandler(console_handler)
    for handler in logger.handlers:
        handler.setLevel(level)

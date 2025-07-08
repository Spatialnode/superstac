from enum import Enum


class AuthType(str, Enum):
    BEARER = "bearer"
    BASIC = "basic"
    API_KEY = "apikey"

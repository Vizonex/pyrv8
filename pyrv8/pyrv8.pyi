from typing import Any

class Context:
    """Synchronous Javascript Runtime Powered by rustyscript and pyo3 written in Rust"""
    def __init__(self, timeout: float | None = ..., max_heap_size: int | None = ...) -> None: ...
    @property
    def timeout(self) -> float:...
    @property
    def current_dir(self) -> str:...
    def set_current_dir(self, path: str) -> None:...
    def advance(self, wait_for_inspector: bool | None = ..., pump_v8_message_loop: bool | None = ...) -> bool:
        """
        Advances eventloop by a single tick this best used
        with python asyncio, uvloop, winloop or rloop.
        This is meant to be used with Javascript Promise Values since
        an asyncio eventloop can call this if it's waiting on a Promise value
        """
    def eval(self, code:str) -> Any:...
    def call(self, name:str, *args) -> Any:...
    def call_module(self, module: "JsHandle", name: str, *args) -> Any:...

    def get_value(self, name:str) -> Any:...
    def load_module(self, module: "JsModule" ) -> "JsHandle":...

    def call_async(self, name:str, *args) -> "JsPromise":...
    def call_module_async(self, module: "JsHandle", name: str, *args) -> "JsPromise":...




class JsPromise:
    def is_done(self) -> bool:
        """
    Returns true if Exception was thrown or a Result came back
    from walking through the eventloop
        """
    
    def step(self, ctx: Context) -> bool:
        """
        Steps a single increment into the eventloop 
        while also checking if the value is finished.
        """
    
    def result(self) -> Any:
        """Obtains result

        :raises InavlidStateError: if state is invalid
        :raises RuntimeError: if JsPromise thrown an error
        """  
    def exception(self) -> RuntimeError | None:
        """Obtains an exception if one was given
        Otherwise this function results with nothing.
        :raises InvalidStateError: if promise did not 
        complete yet"""

class JsModule:
    def __init__(self, filename:str, contents:str) -> None:
        pass

    @staticmethod
    def load(filename: str) -> JsModule:
        """loads a new `JsModule`
        :raises FileNotFoundError: if file was not found
        """
    
    @staticmethod
    def load_dir(directory: str) -> list[JsModule]:
        """loads a directory of javascript and typescript modules.
        raises an exception if a file was not found or an unexpected failure occurs
        """
    
    @property
    def filename(self) -> str:...

    @property
    def contents(self) -> str:...


class JsHandle:
    """Not meant to be initalized in Python but rather in rust 
    and is used for defining a module as being pre-existing with a `Context`
    """
    @property
    def filename(self) -> str:...
    
    @property
    def contents(self) -> str:...



# mimics asyncio.InvalidStateError...
class InvalidStateError(Exception):
    """The operation is not allowed in this state."""


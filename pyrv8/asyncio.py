import asyncio
from .pyrv8 import JsHandle, JsModule, JsPromise, Context as JsContext, InvalidStateError
from typing import Any
from types import coroutine

class Promise(asyncio.Future[Any]):
    __slots__ = ("_ctx", "_p")
    def __init__(self, context: "AsyncContext", p: JsPromise):
        self._ctx = context._ctx
        self._p = p
        self._cancelled = False
        self._cancelled_exc = None

    def set_result(self, result):
        raise RuntimeError('Promise does not support set_result operation')

    def set_exception(self, exception):
        raise RuntimeError('Promise does not support set_exception operation')

    def __step(self):
        self._ctx.advance()
        if self._p.step(self._ctx):
            if exc := self._p.exception():
                super().set_exception(exc)
            else:
                super().set_result(self._p.result())

    def __await__(self):
        if not self.done():
            self._asyncio_future_blocking = True
            self.__step()
            yield self  # This tells Task to wait for completion.
        if not self.done():
            raise RuntimeError("await wasn't used with future")
        return self.result()  # May raise too.




class AsyncContext:
    """Asynchronous Javascript Context for asyncio, uvloop, winloop or rloop"""
    __slots__ = ("_ctx", "_loop", "_promises")
    def __init__(
        self,  
        timeout: float | None = None, 
        max_heap_size: int | None = None,
        loop: asyncio.AbstractEventLoop | None = None
    ) -> None:
        self._ctx = JsContext(timeout, max_heap_size)
        self._loop = loop or asyncio.get_event_loop()
        self._promises: set[Promise] = set()

    @property
    def timeout(self) -> float:
        return self._ctx.timeout
    @property
    def current_dir(self) -> str:
        return self._ctx.current_dir
    
    def set_current_dir(self, path: str) -> None:
        return self._ctx.set_current_dir(path)

    def advance(self, wait_for_inspector: bool | None = None, pump_v8_message_loop: bool | None = None) -> bool:
        """
        Advances eventloop by a single tick this best used
        with python asyncio, uvloop, winloop or rloop.
        This is meant to be used with Javascript Promise Values since
        an asyncio eventloop can call this if it's waiting on a Promise value
        """
        self._ctx.advance(wait_for_inspector, pump_v8_message_loop)

    def eval(self, code:str) -> Any:
        self._ctx.eval(code)

    def call(self, name:str, *args) -> Any:
        return self._ctx.call(name, *args)
    
    def call_module(self, module: "JsHandle", name: str, *args) -> Any:
        return self._ctx.call_module(module, name, *args)

    def get_value(self, name:str) -> Any:
        return self._ctx.get_value(name)

    def load_module(self, module: "JsModule" ) -> "JsHandle":
        return self._ctx.load_module(module)
    
    def __init_promise(self, js:JsPromise, name: str | None) -> Promise:
        p = Promise(self, js, name=name)
        self._promises.add(p)
        p.add_done_callback(self._promises.remove)
        return 
    
    def call_async(self, name:str, *args, fut_name:str | None = None) -> Promise:
        return self.__init_promise(self._ctx.call_async(name, *args), fut_name)

    def call_module_async(self, module: "JsHandle", name: str, *args) -> "JsPromise":
        return self.__init_promise(self._ctx.call_module_async(module, name, *args))

    @coroutine
    def wait(self):
        """waits on lingering futures that might be in the process of resolving"""
        while self._promises:
            self._ctx.advance()
            yield
        yield

    def cancel(self):
        """Cancells all promisses being awaited on"""
        while self._promises:
            f = self._promises.pop()
            if not f.done():
                f.cancel()

    async def __aenter__(self):
        return self
    
    async def __aexit__(self, *args) -> None:
        self.cancel()

    

from pyrv8 import Context



def test_context_eval() -> None:
    context = Context()
    assert context.eval("5+5") == 10

def test_context_eval_with_function_call() -> None:
    context = Context()
    context.eval(
        """
        function add(a, b){
            return a + b;
        }
        """
    )

    assert context.call("add", 1, 2) == 3


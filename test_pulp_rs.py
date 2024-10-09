import pytest
import pulp_rs

def test_optimized_class():
    optimized = pulp_rs.OptimizedClass()
    assert isinstance(optimized, pulp_rs.OptimizedClass)

    optimized.set_value(42)
    assert optimized.get_value() == 42

def test_lp_element():
    # Test creation and name getter
    element = pulp_rs.LpElement("x")
    assert element.name == "x"

    # Test __str__ method
    assert str(element) == "x"

    # Test __pos__ method
    pos_element = +element
    assert pos_element.name == "x"

def test_lp_affine_expression():
    # Test creation with constant and name
    expr = pulp_rs.LpAffineExpression(constant=5.0, name="expr")
    assert expr.constant == 5.0
    assert expr.name == "expr"

    # Test setName and getName methods
    expr.setName("new_expr")
    assert expr.getName() == "new_expr"

if __name__ == "__main__":
    pytest.main([__file__])

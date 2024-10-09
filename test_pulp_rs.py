import pytest
import pulp_rs
import pulp

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

    # Test behaviour matches original library
    assert str(pulp.LpElement("x")) == str(element)

def test_lp_affine_expression():
    # Test creation with constant and name
    expr = pulp_rs.LpAffineExpression(constant=5.0, name="expr")
    assert expr.constant == 5.0
    assert expr.name == "expr"

    # Test setName and getName methods
    expr.setName("new_expr")
    assert expr.name == "new_expr"

    assert (
        pulp.LpAffineExpression(constant=5.0, name="new_expr").name
        == expr.name
    )


def test_lp_affine_str():
    expr = "5.0"
    # Test __str__ method
    assert str(expr) == "5.0"

    # Test with variables
    x = pulp_rs.LpElement("x")
    y = pulp_rs.LpElement("y")
    expr_with_vars = pulp_rs.LpAffineExpression({x: 2, y: 3}, constant=1.5, name="expr_with_vars")
    assert str(expr_with_vars) == "2*x + 3*y + 1.5"

    # Test behaviour matches original library
    pulp_expr = pulp.LpAffineExpression(constant=5.0, name="new_expr")
    assert str(pulp_expr) == str(expr)

    pulp_expr_with_vars = pulp.LpAffineExpression({pulp.LpElement("x"): 2, pulp.LpElement("y"): 3}, constant=1.5, name="expr_with_vars")
    assert str(pulp_expr_with_vars) == str(expr_with_vars)

    assert expr_with_vars.isAtomic() == False
    assert expr_with_vars.isNumericalConstant() == False
    assert str(expr_with_vars.atom()) == str(x)

    assert expr_with_vars.__bool__() == True

    # Python library crashes, '.value' seems a dead code path
    # assert expr_with_vars.value() == pulp_expr_with_vars.value()


@pytest.mark.skip(reason="Unfinished implementation")
def test_var_value_or_default():
    x = pulp_rs.LpElement("x")
    assert x.valueOrDefault() is None

    x.set_value(42)
    assert x.valueOrDefault() == 42

    # Test other classes
    expr = pulp_rs.LpAffineExpression(constant=5.0, name="expr")
    assert expr.valueOrDefault() is None

    expr.set_value(42)
    assert expr.valueOrDefault() == 42

    constraint = pulp_rs.LpConstraint(name="constraint", sense=pulp_rs.Sense.Eq, rhs=10.0)
    assert constraint.valueOrDefault() is None

    constraint.set_value(42)
    assert constraint.valueOrDefault() == 42

    variable = pulp_rs.LpVariable(name="variable", lowBound=0, upBound=10, cat=pulp_rs.LpContinuous)
    assert variable.valueOrDefault() is None

    variable.set_value(42)
    assert variable.valueOrDefault() == 42

if __name__ == "__main__":
    pytest.main([__file__])

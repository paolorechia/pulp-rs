import pulp_rs

def test_optimized_class():
    optimized = pulp_rs.OptimizedClass()
    print("OptimizedClass instance created successfully")

    optimized.set_value(42)
    assert optimized.get_value() == 42
    print(f"Value set and retrieved: {optimized.get_value()}")

def test_lp_element():
    # Test creation and name getter
    element = pulp_rs.LpElement("x1")
    assert element.name == "x1"
    print(f"LpElement created with name: {element.name}")

    # Test __str__ method
    assert str(element) == "x1"
    print(f"LpElement string representation: {str(element)}")

    # Test __pos__ method
    pos_element = +element
    assert pos_element.name == "x1"
    print("__pos__ method works correctly")

def test_lp_affine_expression():
    # Test creation with constant and name
    expr = pulp_rs.LpAffineExpression(constant=5.0, name="expr1")
    assert expr.constant == 5.0
    assert expr.name == "expr1"
    print(f"LpAffineExpression created with constant {expr.constant} and name {expr.name}")

    # Test setName and getName methods
    expr.setName("new_expr")
    assert expr.getName() == "new_expr"
    print(f"LpAffineExpression name changed to: {expr.getName()}")

if __name__ == "__main__":
    test_optimized_class()
    test_lp_element()
    test_lp_affine_expression()
    print("All tests passed successfully!")

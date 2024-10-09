import pulp_rs

# Test the OptimizedClass
optimized = pulp_rs.OptimizedClass()
print("OptimizedClass instance created successfully")

# Test setting and getting a value
optimized.set_value(42)
print(f"Value set and retrieved: {optimized.get_value()}")

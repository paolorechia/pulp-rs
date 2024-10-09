use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::cmp::PartialEq;
use std::fmt;

pub struct LpElement {
    name: Option<String>,
    hash: u64,
    modified: bool,
}

impl LpElement {
    fn new(name: Option<String>) -> Self {
        let hash = std::ptr::addr_of!(name) as u64;
        Self {
            name: name.map(|n| Self::sanitize_name(&n)),
            hash,
            modified: true,
        }
    }

    fn sanitize_name(name: &str) -> String {
        name.replace(&['-', '+', '[', ']', ' ', '-', '>', '/'][..], "_")
    }

    fn set_name(&mut self, name: Option<String>) {
        self.name = name.map(|n| Self::sanitize_name(&n));
    }

    fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Hash for LpElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl fmt::Display for LpElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.as_deref().unwrap_or(""))
    }
}

impl fmt::Debug for LpElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.as_deref().unwrap_or(""))
    }
}

impl Neg for &LpElement {
    type Output = LpAffineExpression;

    fn neg(self) -> Self::Output {
        -LpAffineExpression::from(self)
    }
}

impl Add for &LpElement {
    type Output = LpAffineExpression;

    fn add(self, other: Self) -> Self::Output {
        LpAffineExpression::from(self) + LpAffineExpression::from(other)
    }
}

impl Sub for &LpElement {
    type Output = LpAffineExpression;

    fn sub(self, other: Self) -> Self::Output {
        LpAffineExpression::from(self) - LpAffineExpression::from(other)
    }
}

impl Mul for &LpElement {
    type Output = LpAffineExpression;

    fn mul(self, other: Self) -> Self::Output {
        LpAffineExpression::from(self) * LpAffineExpression::from(other)
    }
}

impl PartialEq for LpElement {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Clone)]
pub struct LpVariable {
    pub name: Option<String>,
    pub low_bound: Option<f64>,
    pub up_bound: Option<f64>,
    pub category: LpCategory,
    pub var_value: Option<f64>,
    pub dj: Option<f64>,
    _lowbound_original: Option<f64>,
    _upbound_original: Option<f64>,
}

impl LpVariable {
    pub fn new(
        name: Option<String>,
        low_bound: Option<f64>,
        up_bound: Option<f64>,
        category: LpCategory,
    ) -> Self {
        let mut var = LpVariable {
            name,
            low_bound,
            up_bound,
            category,
            var_value: None,
            dj: None,
            _lowbound_original: low_bound,
            _upbound_original: up_bound,
        };

        if category == LpCategory::Binary {
            var.low_bound = Some(0.0);
            var.up_bound = Some(1.0);
            var.category = LpCategory::Integer;
        }

        var
    }

    pub fn to_dict(&self) -> HashMap<String, Value> {
        let mut dict = HashMap::new();
        dict.insert("lowBound".to_string(), json!(self.low_bound));
        dict.insert("upBound".to_string(), json!(self.up_bound));
        dict.insert("cat".to_string(), json!(self.category));
        dict.insert("varValue".to_string(), json!(self.var_value));
        dict.insert("dj".to_string(), json!(self.dj));
        dict.insert("name".to_string(), json!(self.name));
        dict
    }

    pub fn from_dict(mut dict: HashMap<String, Value>) -> Result<Self, Box<dyn Error>> {
        let mut var = LpVariable::new(
            dict.remove("name").and_then(|v| v.as_str().map(|s| s.to_string())),
            dict.remove("lowBound").and_then(|v| v.as_f64()),
            dict.remove("upBound").and_then(|v| v.as_f64()),
            dict.remove("cat")
                .and_then(|v| v.as_str())
                .map(LpCategory::from_str)
                .transpose()?
                .unwrap_or(LpCategory::Continuous),
        );
        var.dj = dict.remove("dj").and_then(|v| v.as_f64());
        var.var_value = dict.remove("varValue").and_then(|v| v.as_f64());
        Ok(var)
    }

    pub fn get_lb(&self) -> Option<f64> {
        self.low_bound
    }

    pub fn get_ub(&self) -> Option<f64> {
        self.up_bound
    }

    pub fn bounds(&mut self, low: Option<f64>, up: Option<f64>) {
        self.low_bound = low;
        self.up_bound = up;
    }

    pub fn positive(&mut self) {
        self.bounds(Some(0.0), None);
    }

    pub fn value(&self) -> Option<f64> {
        self.var_value
    }

    pub fn round(&mut self, eps_int: f64, eps: f64) {
        if let Some(value) = self.var_value {
            if let Some(up_bound) = self.up_bound {
                if value > up_bound && value <= up_bound + eps {
                    self.var_value = Some(up_bound);
                }
            }
            if let Some(low_bound) = self.low_bound {
                if value < low_bound && value >= low_bound - eps {
                    self.var_value = Some(low_bound);
                }
            }
            if self.category == LpCategory::Integer && (value.round() - value).abs() <= eps_int {
                self.var_value = Some(value.round());
            }
        }
    }
    pub fn rounded_value(&self, eps: f64) -> f64 {
        if self.category == LpCategory::Integer
            && self.var_value.is_some()
            && (self.var_value.unwrap() - self.var_value.unwrap().round()).abs() <= eps
        {
            self.var_value.unwrap().round()
        } else {
            self.var_value.unwrap_or(0.0)
        }
    }

    pub fn value_or_default(&self) -> f64 {
        if let Some(value) = self.var_value {
            value
        } else if let Some(low_bound) = self.low_bound {
            if let Some(up_bound) = self.up_bound {
                if 0.0 >= low_bound && 0.0 <= up_bound {
                    0.0
                } else if low_bound >= 0.0 {
                    low_bound
                } else {
                    up_bound
                }
            } else if 0.0 >= low_bound {
                0.0
            } else {
                low_bound
            }
        } else if let Some(up_bound) = self.up_bound {
            if 0.0 <= up_bound {
                0.0
            } else {
                up_bound
            }
        } else {
            0.0
        }
    }

    pub fn is_valid(&self, eps: f64) -> bool {
        if self.name.as_deref() == Some("__dummy") && self.var_value.is_none() {
            return true;
        }
        if self.var_value.is_none() {
            return false;
        }
        let value = self.var_value.unwrap();
        if let Some(up_bound) = self.up_bound {
            if value > up_bound + eps {
                return false;
            }
        }
        if let Some(low_bound) = self.low_bound {
            if value < low_bound - eps {
                return false;
            }
        }
        if self.category == LpCategory::Integer && (value.round() - value).abs() > eps {
            return false;
        }
        true
    }

    pub fn infeasibility_gap(&self, mip: bool) -> Result<f64, Box<dyn Error>> {
        let value = self.var_value.ok_or("variable value is None")?;
        if let Some(up_bound) = self.up_bound {
            if value > up_bound {
                return Ok(value - up_bound);
            }
        }
        if let Some(low_bound) = self.low_bound {
            if value < low_bound {
                return Ok(value - low_bound);
            }
        }
        if mip && self.category == LpCategory::Integer && value.round() - value != 0.0 {
            return Ok(value.round() - value);
        }
        Ok(0.0)
    }

    pub fn is_binary(&self) -> bool {
        self.category == LpCategory::Integer && self.low_bound == Some(0.0) && self.up_bound == Some(1.0)
    }

    pub fn is_integer(&self) -> bool {
        self.category == LpCategory::Integer
    }

    pub fn is_free(&self) -> bool {
        self.low_bound.is_none() && self.up_bound.is_none()
    }

    pub fn is_constant(&self) -> bool {
        self.low_bound.is_some() && self.up_bound == self.low_bound
    }

    pub fn is_positive(&self) -> bool {
        self.low_bound == Some(0.0) && self.up_bound.is_none()
    }

    pub fn as_cplex_lp_variable(&self) -> String {
        if self.is_free() {
            format!("{} free", self.name.as_deref().unwrap_or(""))
        } else if self.is_constant() {
            format!("{} = {:.12}", self.name.as_deref().unwrap_or(""), self.low_bound.unwrap())
        } else {
            let mut s = String::new();
            if let Some(low_bound) = self.low_bound {
                if low_bound == 0.0 && self.category == LpCategory::Continuous {
                    // Do nothing
                } else {
                    s.push_str(&format!("{:.12} <= ", low_bound));
                }
            } else {
                s.push_str("-inf <= ");
            }
            s.push_str(self.name.as_deref().unwrap_or(""));
            if let Some(up_bound) = self.up_bound {
                s.push_str(&format!(" <= {:.12}", up_bound));
            }
            s
        }
    }

    pub fn set_initial_value(&mut self, val: f64, check: bool) -> Result<bool, Box<dyn Error>> {
        let lb = self.low_bound;
        let ub = self.up_bound;

        if let Some(lb) = lb {
            if val < lb {
                if !check {
                    return Ok(false);
                }
                return Err(format!("In variable {}, initial value {} is smaller than lowBound {}", 
                    self.name.as_deref().unwrap_or(""), val, lb).into());
            }
        }

        if let Some(ub) = ub {
            if val > ub {
                if !check {
                    return Ok(false);
                }
                return Err(format!("In variable {}, initial value {} is greater than upBound {}", 
                    self.name.as_deref().unwrap_or(""), val, ub).into());
            }
        }

        self.var_value = Some(val);
        Ok(true)
    }

    pub fn fix_value(&mut self) {
        if let Some(val) = self.var_value {
            self.bounds(Some(val), Some(val));
        }
    }

    pub fn unfix_value(&mut self) {
        self.bounds(self._lowbound_original, self._upbound_original);
    }

}

impl LpElement for LpVariable {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LpCategory {
    Continuous,
    Integer,
    Binary,
}

impl FromStr for LpCategory {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Continuous" => Ok(LpCategory::Continuous),
            "Integer" => Ok(LpCategory::Integer),
            "Binary" => Ok(LpCategory::Binary),
            _ => Err("Invalid category".into()),
        }
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct LpAffineExpression {
    constant: f64,
    name: Option<String>,
    terms: HashMap<LpVariable, f64>,
}

impl LpAffineExpression {
    pub fn new(e: Option<impl Into<LpAffineExpression>>, constant: f64, name: Option<String>) -> Self {
        let mut expr = match e {
            Some(e) => e.into(),
            None => LpAffineExpression {
                constant,
                name,
                terms: HashMap::new(),
            },
        };
        expr.constant += constant;
        expr.name = name;
        expr
    }

    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name.map(|n| n.replace(&['-', '+', '[', ']', ' '][..], "_"));
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn is_atomic(&self) -> bool {
        self.terms.len() == 1 && self.constant == 0.0 && self.terms.values().next().unwrap() == &1.0
    }

    pub fn is_numerical_constant(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn atom(&self) -> Option<&LpVariable> {
        self.terms.keys().next()
    }

    pub fn value(&self) -> Option<f64> {
        let mut sum = self.constant;
        for (v, &x) in &self.terms {
            match v.var_value {
                Some(val) => sum += val * x,
                None => return None,
            }
        }
        Some(sum)
    }

    pub fn value_or_default(&self) -> f64 {
        let mut sum = self.constant;
        for (v, &x) in &self.terms {
            sum += v.value_or_default() * x;
        }
        sum
    }

    pub fn add_term(&mut self, key: LpVariable, value: f64) {
        *self.terms.entry(key).or_insert(0.0) += value;
    }

    pub fn empty_copy(&self) -> Self {
        LpAffineExpression {
            constant: 0.0,
            name: None,
            terms: HashMap::new(),
        }
    }

    pub fn sorted_keys(&self) -> Vec<&LpVariable> {
        let mut keys: Vec<_> = self.terms.keys().collect();
        keys.sort_by_key(|v| v.name());
        keys
    }

    fn count_characters(line: &[String]) -> usize {
        // counts the characters in a list of strings
        line.iter().map(|s| s.len()).sum()
    }

    pub fn as_cplex_variables_only(&self, name: &str) -> (Vec<String>, Vec<String>) {
        let mut result = Vec::new();
        let mut line = vec![format!("{}:", name)];
        let mut not_first = false;
        let variables = self.sorted_keys();

        for v in variables {
            let val = self.terms.get(v).unwrap();
            let (sign, val) = if *val < 0.0 {
                (" -", -val)
            } else if not_first {
                (" +", *val)
            } else {
                ("", *val)
            };
            not_first = true;

            let term = if val == 1.0 {
                format!("{} {}", sign, v.name())
            } else {
                // adding zero to val to remove instances of negative zero
                format!("{} {:.12} {}", sign, val + 0.0, v.name())
            };

            if Self::count_characters(&line) + term.len() > crate::const::LP_CPLEX_LP_LINE_SIZE {
                result.push(line.join(""));
                line = vec![term];
            } else {
                line.push(term);
            }
        }
        (result, line)
    }

    pub fn as_cplex_lp_affine_expression(&self, name: &str, constant: bool) -> String {
        let (mut result, mut line) = self.as_cplex_variables_only(name);

        let term = if self.terms.is_empty() {
            format!(" {}", self.constant)
        } else if constant {
            match self.constant {
                c if c < 0.0 => format!(" - {}", -c),
                c if c > 0.0 => format!(" + {}", c),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        if Self::count_characters(&line) + term.len() > crate::const::LP_CPLEX_LP_LINE_SIZE {
            result.push(line.join(""));
            line = vec![term];
        } else {
            line.push(term);
        }
        result.push(line.join(""));
        result.join("\n") + "\n"
    }
    // Other methods like `as_cplex_variables_only`, `as_cplex_lp_affine_expression`, etc.
    // can be implemented similarly, adapting them to Rust's string handling and formatting.

    pub fn add_in_place(&mut self, other: &LpAffineExpression, sign: f64) {
        self.constant += other.constant * sign;
        for (v, &x) in &other.terms {
            self.add_term(v.clone(), x * sign);
        }
    }

    pub fn sub_in_place(&mut self, other: &LpAffineExpression) {
        self.add_in_place(other, -1.0);
    }
}

impl Neg for LpAffineExpression {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut e = self.empty_copy();
        e.constant = -self.constant;
        for (v, &x) in &self.terms {
            e.terms.insert(v.clone(), -x);
        }
        e
    }
}

impl Add for &LpAffineExpression {
    type Output = LpAffineExpression;

    fn add(self, other: &LpAffineExpression) -> Self::Output {
        let mut result = self.clone();
        result.add_in_place(other, 1.0);
        result
    }
}

impl Sub for &LpAffineExpression {
    type Output = LpAffineExpression;

    fn sub(self, other: &LpAffineExpression) -> Self::Output {
        let mut result = self.clone();
        result.sub_in_place(other);
        result
    }
}

impl Mul<f64> for &LpAffineExpression {
    type Output = LpAffineExpression;

    fn mul(self, other: f64) -> Self::Output {
        let mut e = self.empty_copy();
        e.constant = self.constant * other;
        for (v, &x) in &self.terms {
            e.terms.insert(v.clone(), x * other);
        }
        e
    }
}

impl Div<f64> for &LpAffineExpression {
    type Output = LpAffineExpression;

    fn div(self, other: f64) -> Self::Output {
        let mut e = self.empty_copy();
        e.constant = self.constant / other;
        for (v, &x) in &self.terms {
            e.terms.insert(v.clone(), x / other);
        }
        e
    }
}

// Implement PartialOrd for comparison operations (<=, >=, ==)
impl PartialOrd for LpAffineExpression {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // This is a placeholder implementation. In practice, you'd need to
        // implement this based on your specific requirements for comparing
        // LpAffineExpressions.
        None
    }
}

impl LpAffineExpression {
    pub fn to_dict(&self) -> Vec<HashMap<String, String>> {
        self.terms
            .iter()
            .map(|(k, v)| {
                let mut map = HashMap::new();
                map.insert("name".to_string(), k.name().unwrap_or("").to_string());
                map.insert("value".to_string(), v.to_string());
                map
            })
            .collect()
    }
}
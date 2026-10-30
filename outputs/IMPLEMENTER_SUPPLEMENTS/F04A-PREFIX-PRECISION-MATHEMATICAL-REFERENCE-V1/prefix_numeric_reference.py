"""Independent exact-rational F04A.4 reference. Synthetic inputs only."""
import argparse
import ast
import hashlib
import itertools
import json
import math
import struct
from fractions import Fraction
from pathlib import Path

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def power_two(exponent):
    return Fraction(2) ** exponent

def value(word, precision=24, exponent_bits=8):
    fraction_bits = precision - 1
    bias = (1 << exponent_bits - 1) - 1
    exponent = word >> fraction_bits & (1 << exponent_bits) - 1
    mantissa = word & (1 << fraction_bits) - 1
    require(exponent != (1 << exponent_bits) - 1, 'nonfinite input')
    result = Fraction(mantissa if exponent == 0 else mantissa + (1 << fraction_bits))
    result *= power_two((1 - bias if exponent == 0 else exponent - bias) - fraction_bits)
    return -result if word >> fraction_bits + exponent_bits else result

def round_integer(number):
    quotient, remainder = divmod(number.numerator, number.denominator)
    doubled = remainder * 2
    return quotient + int(doubled > number.denominator or (doubled == number.denominator and quotient % 2))

def round_word(number, precision=24, exponent_bits=8, negative_zero=False):
    sign = int(number < 0 or (number == 0 and negative_zero))
    number = abs(number)
    shift = precision - 1 + exponent_bits
    if number == 0:
        return sign << shift
    exponent = number.numerator.bit_length() - number.denominator.bit_length()
    if number < power_two(exponent):
        exponent -= 1
    bias = (1 << exponent_bits - 1) - 1
    minimum = 1 - bias
    unit = power_two(max(exponent, minimum) - precision + 1)
    significand = round_integer(number / unit)
    if significand >= 1 << precision:
        significand >>= 1
        exponent += 1
    require(exponent <= bias, 'overflow')
    if exponent < minimum and significand < 1 << precision - 1:
        return sign << shift | significand
    exponent = max(exponent, minimum)
    return sign << shift | exponent + bias << precision - 1 | significand - (1 << precision - 1)

def multiply(left, right):
    return round_word(value(left) * value(right), negative_zero=bool((left ^ right) >> 31))

def add(left, right):
    return round_word(value(left) + value(right), negative_zero=left == right == 2147483648)

def sqrt_word(number, precision, exponent_bits):
    """Binary search adjacent finite floats, then compare exact squared midpoint."""
    require(number > 0, 'positive sqrt')
    low, high = (0, (1 << exponent_bits) - 1 << precision - 1)
    while low + 1 < high:
        middle = (low + high) // 2
        if value(middle, precision, exponent_bits) ** 2 <= number:
            low = middle
        else:
            high = middle
    lower, upper = (value(low, precision, exponent_bits), value(high, precision, exponent_bits))
    midpoint_squared = ((lower + upper) / 2) ** 2
    return high if number > midpoint_squared or (number == midpoint_squared and low % 2) else low

def constants():
    records = []
    for literal in ('0.85', '0.15'):
        literal64 = round_word(Fraction(literal), 53, 11)
        root64 = sqrt_word(value(literal64, 53, 11), 53, 11)
        root32 = round_word(value(root64, 53, 11))
        wrong_input32 = round_word(Fraction(literal))
        wrong_root64 = sqrt_word(value(wrong_input32), 53, 11)
        wrong_root32 = round_word(value(wrong_root64, 53, 11))
        records.append({'literal': literal, 'binary64_literal_u64': f'{literal64:016x}', 'sqrt_binary64_u64': f'{root64:016x}', 'sqrt_then_f32_u32': f'{root32:08x}', 'f32_input_then_sqrt_u32': f'{wrong_root32:08x}'})
    return records

def mix_sample(initial, source, filtered, gain, mode, active, weights):
    if active:
        coefficient_a = multiply(weights[0], gain)
        coefficient_b = multiply(multiply(weights[1], round_word(Fraction(mode))), gain)
    else:
        coefficient_a, coefficient_b = (gain, 0)
    product_a = multiply(source, coefficient_a)
    partial = add(product_a, initial)
    product_b = multiply(filtered, coefficient_b) if active else None
    result = add(partial, product_b) if active else partial
    return (result, (coefficient_a, coefficient_b, product_a, partial, product_b))

def samples(recipe, length):
    if recipe == 'alternating_dyadic':
        return [round_word(Fraction(i % 17 - 8, 1 << 18) * (-1 if i % 2 else 1)) for i in range(length)]
    if recipe == 'boundary_impulses':
        return [round_word(Fraction(-1 if i % 2 else 1, 1 << 12)) if i in (0, 1, 15, 16, 63, 64, 1535) else 0 for i in range(length)]
    if recipe == 'signed_zero_subnormal':
        return [[0, 2147483648, 1, 2147483649, 8388607, 2155872255][i % 6] for i in range(length)]
    if recipe == 'zero':
        return [0] * length
    raise RuntimeError('unknown synthetic recipe')

def run():
    records = constants()
    weights = [int(row['sqrt_then_f32_u32'], 16) for row in records]
    early_weights = [int(row['f32_input_then_sqrt_u32'], 16) for row in records]
    root64 = [value(int(row['sqrt_binary64_u64'], 16), 53, 11) for row in records]
    comparisons = differences = early_differences = total = 0
    discriminating_input = None
    trace_fields = ['coefficient_a_u32', 'coefficient_b_u32', 'source_product_u32', 'first_sum_u32', 'filter_product_u32', 'result_u32']
    recipes = ('alternating_dyadic', 'boundary_impulses', 'signed_zero_subnormal')
    gains = [round_word(Fraction(1)), round_word(Fraction(3, 10)), round_word(Fraction(7, 10)), round_word(Fraction(1, 1000000))]
    for recipe, gain, mode, active in itertools.product(recipes, gains, (-1, 0, 1), (False, True)):
        length = 17
        initial = samples('alternating_dyadic', length)
        source = samples(recipe, length)
        filtered = samples('boundary_impulses', length)
        result = []
        for index, (initial_word, source_word, filter_word) in enumerate(zip(initial, source, filtered)):
            correct, _ = mix_sample(initial_word, source_word, filter_word, gain, mode, active, weights)
            early, _ = mix_sample(initial_word, source_word, filter_word, gain, mode, active, early_weights)
            if active:
                coefficient_a = round_word(value(round_word(root64[0] * value(gain), 53, 11), 53, 11))
                intermediate_b = value(round_word(root64[1] * mode, 53, 11), 53, 11)
                coefficient_b = round_word(value(round_word(intermediate_b * value(gain), 53, 11), 53, 11))
                incorrect = add(add(multiply(source_word, coefficient_a), initial_word), multiply(filter_word, coefficient_b))
            else:
                incorrect = correct
            result.append(correct)
            differences += correct != incorrect
            if correct != incorrect and discriminating_input is None:
                discriminating_input = {'recipe': recipe, 'gain_u32': f'{gain:08x}', 'mode': mode, 'filtered_branch': active, 'index': index, 'initial_u32': f'{initial_word:08x}', 'source_u32': f'{source_word:08x}', 'filter_input_u32': f'{filter_word:08x}'}
            early_differences += correct != early
            total += 1
    previous_pcm = samples('zero', 1536)
    previous_filter = samples('zero', 1536)
    fresh_previous_differs_from_stale = 0
    prior_current = [0] * 1536
    for generation, gain in enumerate(gains[:3]):
        current_pcm = samples(recipes[generation], 1536)
        current_filter = samples('boundary_impulses', 1536)
        for pcm, filters, previous in [(previous_pcm, previous_filter, True), (current_pcm, current_filter, False)]:
            rendered = [mix_sample(0, x, f, gain, -1, True, weights)[0] for x, f in zip(pcm, filters)]
            if previous and generation:
                fresh_previous_differs_from_stale += sum((a != b for a, b in zip(rendered, prior_current)))
            if not previous:
                prior_current = rendered
            total += 1536
        previous_pcm, previous_filter = (current_pcm, current_filter)
    require(differences > 0, 'omitted pre-gain F32 rounding mutation must be detected')
    require(early_differences == 0 and weights == early_weights, 'literal-input rounding equivalence')
    require(fresh_previous_differs_from_stale > 0, 'stale previous-prefix mutation must be detected')
    require(round_word(Fraction(1) + power_two(-24)) == 1065353216, 'tie even')
    require(round_word(Fraction(1) + 3 * power_two(-24)) == 1065353218, 'tie odd')
    require(multiply(1, 1056964608) == 0, 'subnormal tie to zero')
    require(multiply(3, 1056964608) == 2, 'subnormal tie to even')
    require(add(2147483648, 2147483648) == 2147483648, 'negative zero add')
    require(add(1065353216, 3212836864) == 0, 'cancellation')
    require(multiply(2147483648, 1065353216) == 2147483648, 'negative zero multiply')
    return {'status': 'PASS', 'constant_derivation': records, 'synthetic_samples_computed': total, 'frozen_scalar_sample_comparisons': comparisons, 'omitted_pre_gain_f32_output_differences': differences, 'early_f32_literal_output_differences': early_differences, 'discriminating_input_only': discriminating_input, 'stale_previous_prefix_differences': fresh_previous_differs_from_stale, 'trace_fields': trace_fields, 'real_audio': 'NOT_READ', 'native_execution': 'NOT_RUN', 'scope': 'Exact F04A.4 arithmetic and synthetic half recomputation; not whole g0-g2 spatial/DSP conformance.'}

if __name__ == "__main__":
    print(json.dumps(run(), indent=2))

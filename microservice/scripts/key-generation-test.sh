#!/bin/bash

# Key Generation API Test Script
# This script tests the /generate-keys endpoint with various scenarios

# Configuration
API_URL="http://localhost:8080/generate-keys"  
TIMEOUT=10                                     # Timeout in seconds for curl requests
NUM_PARALLEL_REQUESTS=7                        # Number of parallel requests to make in load tests
TOTAL_REQUESTS=49                              # Total number of requests for load test

# Color codes for better readability
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create a temp directory for test outputs
TEMP_DIR=$(mktemp -d)
SUMMARY_FILE="${TEMP_DIR}/summary.txt"
echo "Test results will be stored in ${TEMP_DIR}"

# Initialize the summary file
echo "TEST SUMMARY" > "$SUMMARY_FILE"
echo "===========" >> "$SUMMARY_FILE"

# Function to validate JSON response - expects success
validate_success_response() {
    local response=$1
    local test_name=$2
    local result_file=$3
    local success=true
    
    # Write the original response to the result file
    echo "RESPONSE:" > "$result_file"
    echo "$response" >> "$result_file"
    echo "" >> "$result_file"
    
    # Check if response is valid JSON
    if ! echo "$response" | jq . > /dev/null 2>&1; then
        echo -e "${RED}[FAIL]${NC} $test_name: Invalid JSON response"
        echo "RESULT: [FAIL] - Invalid JSON response" >> "$result_file"
        echo "$test_name: FAIL - Invalid JSON response" >> "$SUMMARY_FILE"
        return 1
    fi
    
    # Validate required fields exist
    local fields=("user_id" "encrypted_private_key" "encryption_salt" "encryption_nonce")
    for field in "${fields[@]}"; do
        if ! echo "$response" | jq -e ".$field" > /dev/null 2>&1; then
            echo -e "${RED}[FAIL]${NC} $test_name: Missing required field '$field'"
            echo "RESULT: [FAIL] - Missing required field '$field'" >> "$result_file"
            echo "$test_name: FAIL - Missing required field '$field'" >> "$SUMMARY_FILE"
            return 1
        fi
    done
    
    echo -e "${GREEN}[PASS]${NC} $test_name: Response schema valid"
    echo "RESULT: [PASS] - Response schema valid" >> "$result_file"
    echo "$test_name: PASS" >> "$SUMMARY_FILE"
    return 0
}

validate_failure_response() {
    local response=$1
    local status_code=$2  # HTTP status code
    local test_name=$3
    local result_file=$4
    local expected_error=$5  # Optional expected error message

    # Write the original response to the result file
    echo "RESPONSE (Status: $status_code):" > "$result_file"
    echo "$response" >> "$result_file"
    echo "" >> "$result_file"

    # For error tests, success is when the API returns a 4xx status code
    if [[ $status_code -ge 400 && $status_code -lt 500 ]]; then
        echo -e "${GREEN}[EXPECTED FAIL]${NC} $test_name: Got $status_code as expected"
        echo "RESULT: [EXPECTED FAIL] - Got $status_code as expected" >> "$result_file"
        echo "$test_name: PASS (Expected failure)" >> "$SUMMARY_FILE"

        # Check for expected error message if provided
        if [ -n "$expected_error" ]; then
            local error_msg=$(echo "$response" | jq -r '.message // "No message field found"')
            if [[ "$error_msg" == *"$expected_error"* ]]; then
                echo -e "${GREEN}[MESSAGE MATCH]${NC} Error contains expected text: '$expected_error'"
                echo "MESSAGE MATCH: Error contains expected text: '$expected_error'" >> "$result_file"
            else
                echo -e "${YELLOW}[WARNING]${NC} Expected error to contain: '$expected_error'"
                echo "WARNING: Expected error to contain: '$expected_error'" >> "$result_file"
            fi
        fi

        return 0
    else
        # We didn't get an error status code but expected one
        echo -e "${RED}[UNEXPECTED PASS]${NC} $test_name: Expected 4xx status but got $status_code"
        echo "RESULT: [UNEXPECTED PASS] - Expected 4xx status but got $status_code" >> "$result_file"
        echo "$test_name: FAIL (Unexpected success)" >> "$SUMMARY_FILE"
        return 1
    fi
}

# Function to run a single test that should pass
run_success_test() {
    local test_name=$1
    local payload=$2
    local result_file="${TEMP_DIR}/${test_name// /_}.txt"
    
    echo -e "\n${YELLOW}Running Test:${NC} $test_name (Expecting Success)"
    echo "Payload: $payload"
    
    # Execute the curl command
    local start_time=$(date +%s.%N)
    local response=$(curl -s -X POST "${API_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        --max-time $TIMEOUT)
    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)
    
    echo "Response time: ${elapsed}s" | tee -a "$result_file"
    validate_success_response "$response" "$test_name" "$result_file"
    
    return $?
}

run_failure_test() {
    local test_name=$1
    local payload=$2
    local expected_error=$3  # Optional
    local result_file="${TEMP_DIR}/${test_name// /_}.txt"

    echo -e "\n${YELLOW}Running Test:${NC} $test_name (Expecting Failure)"
    echo "Payload: $payload"

    # Execute the curl command with -w option to get the status code
    local start_time=$(date +%s.%N)
    local response_and_status=$(curl -s -X POST "${API_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        -w "\n%{http_code}" \
        --max-time $TIMEOUT)
    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)

    # Extract the response body and status code
    local status_code=$(echo "$response_and_status" | tail -n1)
    local response=$(echo "$response_and_status" | sed '$d')

    echo "Response time: ${elapsed}s" | tee -a "$result_file"
    echo "Status code: $status_code" | tee -a "$result_file"
    
    # Make sure status_code is numeric before calling validate_failure_response
    if [[ "$status_code" =~ ^[0-9]+$ ]]; then
        validate_failure_response "$response" "$status_code" "$test_name" "$result_file" "$expected_error"
        return $?
    else
        echo -e "${RED}[ERROR]${NC} Could not determine status code. Got: $status_code"
        echo "RESULT: [ERROR] - Could not determine status code. Got: $status_code" >> "$result_file"
        echo "$test_name: FAIL (Could not determine status code)" >> "$SUMMARY_FILE"
        return 1
    fi
}

# Function to run HTTP method test (should fail)
run_method_test() {
    local test_name=$1
    local method=$2
    local result_file="${TEMP_DIR}/${test_name// /_}.txt"

    echo -e "\n${YELLOW}Running Test:${NC} $test_name (Expecting Failure)"

    # Execute the curl command with specified method and capture status code
    local start_time=$(date +%s.%N)
    local response_and_status=$(curl -s -X "$method" "${API_URL}" \
        -w "\n%{http_code}" \
        --max-time $TIMEOUT)
    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)

    # Extract the status code from the last line
    local status_code=$(echo "$response_and_status" | tail -n1)
    # Extract the response body by removing the last line
    local response=$(echo "$response_and_status" | sed '$d')

    echo "Response time: ${elapsed}s" | tee -a "$result_file"
    echo "Status code: $status_code" | tee -a "$result_file"
    
    # Make sure status_code is numeric
    if [[ "$status_code" =~ ^[0-9]+$ ]]; then
        validate_failure_response "$response" "$status_code" "$test_name" "$result_file" ""
    else
        echo -e "${RED}[ERROR]${NC} Could not determine status code. Got: $status_code"
        echo "RESULT: [ERROR] - Could not determine status code. Got: $status_code" >> "$result_file"
        echo "$test_name: FAIL (Could not determine status code)" >> "$SUMMARY_FILE"
        return 1
    fi

    return $?
}

# Function to run content type test (should fail)
run_content_type_test() {
    local test_name=$1
    local content_type=$2
    local payload=$3
    local result_file="${TEMP_DIR}/${test_name// /_}.txt"

    echo -e "\n${YELLOW}Running Test:${NC} $test_name (Expecting Failure)"

    # Execute the curl command with specified content type and capture status code
    local start_time=$(date +%s.%N)
    local response_and_status=$(curl -s -X POST "${API_URL}" \
        -H "Content-Type: $content_type" \
        -d "$payload" \
        -w "\n%{http_code}" \
        --max-time $TIMEOUT)
    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)

    # Extract the status code from the last line
    local status_code=$(echo "$response_and_status" | tail -n1)
    # Extract the response body by removing the last line
    local response=$(echo "$response_and_status" | sed '$d')

    echo "Response time: ${elapsed}s" | tee -a "$result_file"
    echo "Status code: $status_code" | tee -a "$result_file"
    
    # Make sure status_code is numeric
    if [[ "$status_code" =~ ^[0-9]+$ ]]; then
        validate_failure_response "$response" "$status_code" "$test_name" "$result_file" ""
    else
        echo -e "${RED}[ERROR]${NC} Could not determine status code. Got: $status_code"
        echo "RESULT: [ERROR] - Could not determine status code. Got: $status_code" >> "$result_file"
        echo "$test_name: FAIL (Could not determine status code)" >> "$SUMMARY_FILE"
        return 1
    fi

    return $?
}

# Test Cases

# Test 1: Basic key generation with keyphrase only
test_basic_keyphrase() {
    local payload='{"keyphrase": "strong-secure-passphrase"}'
    run_success_test "Basic keyphrase only" "$payload"
}

# Test 2: Key generation with custom user ID
test_with_custom_user_id() {
    local payload='{"custom_user_id": "user123", "keyphrase": "strong-secure-passphrase"}'
    run_success_test "Custom user ID" "$payload"
}

# Test 3: Using a very long keyphrase
test_long_keyphrase() {
    local long_phrase=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 100)
    local payload="{\"keyphrase\": \"$long_phrase\"}"
    run_success_test "Long keyphrase (100 chars)" "$payload"
}

# Test 4: Using a short keyphrase (should fail)
test_short_keyphrase() {
    local payload='{"keyphrase": "abc"}'
    run_failure_test "Short keyphrase (3 chars)" "$payload" "Keyphrase must be at least 8 characters long"
}

# Test 5: Special characters in keyphrase
test_special_chars() {
    local payload="{\"keyphrase\": \"!@#$%^&*()_+{}|:<>?~\"}"
    run_success_test "Special characters in keyphrase" "$payload"
}

# Test 6: Unicode characters in keyphrase
test_unicode_keyphrase() {
    local payload='{"keyphrase": "パスワード你好こんにちは"}'
    run_success_test "Unicode characters in keyphrase" "$payload"
}

# Test 7: Empty keyphrase (should fail)
test_empty_keyphrase() {
    local payload='{"keyphrase": ""}'
    run_failure_test "Empty keyphrase" "$payload" "Keyphrase must be at least 8 characters long"
}

# Test 8: Missing keyphrase field (should fail)
test_missing_keyphrase() {
    local payload='{}'
    run_failure_test "Missing keyphrase field" "$payload" "parse error"
}

# Test 9: Malformed JSON (should fail)
test_malformed_json() {
    local payload='{"keyphrase": "test", "custom_user_id": '
    run_failure_test "Malformed JSON" "$payload" "parse error"
}

# Test 10: Idempotency - Generate keys twice with same custom user ID
test_idempotency() {
    local user_id="idem_$(date +%s)"
    local payload="{\"custom_user_id\": \"$user_id\", \"keyphrase\": \"test-passphrase\"}"
    local first_result_file="${TEMP_DIR}/idempotency_first.txt"
    local second_result_file="${TEMP_DIR}/idempotency_second.txt"
    
    echo -e "\n${YELLOW}Running Test:${NC} Idempotency - First request"
    local first_response=$(curl -s -X POST "${API_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        --max-time $TIMEOUT)
    
    echo "RESPONSE:" > "$first_result_file"
    echo "$first_response" >> "$first_result_file"
    
    if ! echo "$first_response" | jq -e '.user_id' > /dev/null 2>&1; then
        echo -e "${RED}[FAIL]${NC} Idempotency - First request failed"
        echo "RESULT: [FAIL] - First request failed" >> "$first_result_file"
        echo "Idempotency: FAIL - First request failed" >> "$SUMMARY_FILE"
        return 1
    fi
    
    local first_user_id=$(echo "$first_response" | jq -r '.user_id')
    echo "First user_id: $first_user_id"
    
    echo -e "\n${YELLOW}Running Test:${NC} Idempotency - Second request with same custom_user_id"
    local second_response=$(curl -s -X POST "${API_URL}" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        --max-time $TIMEOUT)
    
    echo "RESPONSE:" > "$second_result_file"
    echo "$second_response" >> "$second_result_file"
    
    if ! echo "$second_response" | jq -e '.user_id' > /dev/null 2>&1; then
        echo -e "${RED}[FAIL]${NC} Idempotency - Second request failed"
        echo "RESULT: [FAIL] - Second request failed" >> "$second_result_file"
        echo "Idempotency: FAIL - Second request failed" >> "$SUMMARY_FILE"
        return 1
    fi
    
    local second_user_id=$(echo "$second_response" | jq -r '.user_id')
    echo "Second user_id: $second_user_id"
    
    if [ "$first_user_id" == "$second_user_id" ]; then
        echo -e "${RED}[FAIL]${NC} Idempotency: Both requests returned the same user_id"
        echo "RESULT: [FAIL] - Both requests returned the same user_id" >> "$second_result_file"
        echo "Idempotency: FAIL - Both requests returned the same user_id" >> "$SUMMARY_FILE"
        return 1
    else
        echo -e "${GREEN}[PASS]${NC} Idempotency: Different user_ids returned"
        echo "First: $first_user_id"
        echo "Second: $second_user_id"
        echo "RESULT: [PASS] - Different user_ids returned (First: $first_user_id, Second: $second_user_id)" >> "$second_result_file"
        echo "Idempotency: PASS - Different user_ids returned" >> "$SUMMARY_FILE"
        return 0
    fi
}

# Test 11: HTTP method test
test_http_method() {
    run_method_test "Wrong HTTP method (GET)" "GET"
}

# Test 12: Content-Type header test
test_content_type() {
    local payload='{"keyphrase": "content-type-test"}'
    run_content_type_test "Wrong Content-Type" "text/plain" "$payload"
}

# Test 13: Load test - Multiple parallel requests
test_parallel_requests() {
    echo -e "\n${YELLOW}Running Test:${NC} Parallel requests ($NUM_PARALLEL_REQUESTS concurrent)"
    
    # Create a function for a single request in the load test
    parallel_request() {
        local id=$1
        local result_file="${TEMP_DIR}/parallel_${id}.txt"
        local payload="{\"keyphrase\": \"parallel-test-$id\"}"
        
        local response=$(curl -s -X POST "${API_URL}" \
            -H "Content-Type: application/json" \
            -d "$payload" \
            --max-time $TIMEOUT)
        
        echo "RESPONSE:" > "$result_file"
        echo "$response" >> "$result_file"
        
        if validate_success_response "$response" "Parallel request $id" "$result_file" > /dev/null; then
            echo -e "${GREEN}[PASS]${NC} Parallel request $id completed successfully"
            return 0
        else
            echo -e "${RED}[FAIL]${NC} Parallel request $id failed"
            return 1
        fi
    }
    
    # Run parallel requests
    local pass_count=0
    local fail_count=0
    
    for ((i=1; i<=$TOTAL_REQUESTS; i+=$NUM_PARALLEL_REQUESTS)); do
        pids=()
        status=()
        
        # Start parallel_requests processes
        end=$((i + NUM_PARALLEL_REQUESTS - 1))
        if [ $end -gt $TOTAL_REQUESTS ]; then
            end=$TOTAL_REQUESTS
        fi
        
        for ((j=i; j<=end; j++)); do
            parallel_request $j &
            pids+=($!)
        done
        
        # Wait for all parallel requests to complete and collect status
        for pid in "${pids[@]}"; do
            wait $pid
            status+=($?)
        done
        
        # Count successes and failures
        for s in "${status[@]}"; do
            if [ $s -eq 0 ]; then
                ((pass_count++))
            else
                ((fail_count++))
            fi
        done
    done
    
    echo -e "\n${YELLOW}Parallel Test Results:${NC}"
    echo -e "${GREEN}Passed: $pass_count${NC} / ${RED}Failed: $fail_count${NC} (Total: $TOTAL_REQUESTS)"
    echo "Parallel requests: PASS=$pass_count, FAIL=$fail_count (Total: $TOTAL_REQUESTS)" >> "$SUMMARY_FILE"
    
    if [ $fail_count -eq 0 ]; then
        return 0
    else
        return 1
    fi
}

# Run all tests

echo "======================================"
echo "Starting key generation API tests"
echo "======================================"
echo "API URL: $API_URL"
echo "Tests will timeout after $TIMEOUT seconds"
echo "======================================"

TEST_RESULTS=()

# Run tests and store results
run_test() {
    "$1"
    TEST_RESULTS+=("$1:$?")
}

# Basic functionality tests
run_test test_basic_keyphrase
run_test test_with_custom_user_id
run_test test_long_keyphrase
run_test test_short_keyphrase
run_test test_special_chars
run_test test_unicode_keyphrase

# Error case tests
run_test test_empty_keyphrase
run_test test_missing_keyphrase
run_test test_malformed_json

# Advanced tests
run_test test_idempotency
run_test test_http_method
run_test test_content_type

# Performance test
run_test test_parallel_requests

# Generate final summary
echo -e "\n======================================"
echo "TEST SUMMARY"
echo "======================================"

PASS_COUNT=0
FAIL_COUNT=0

for result in "${TEST_RESULTS[@]}"; do
    test_name=${result%:*}
    test_status=${result#*:}
    
    # Extract just the function name without "test_"
    display_name=${test_name#test_}
    # Replace underscores with spaces and capitalize first letter
    display_name=$(echo "$display_name" | sed 's/_/ /g' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
    
    if [ "$test_status" -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $display_name"
        ((PASS_COUNT++))
    else
        echo -e "${RED}✗ FAIL${NC}: $display_name"
        ((FAIL_COUNT++))
    fi
done

echo -e "\n======================================"
echo -e "RESULTS: ${GREEN}$PASS_COUNT passed${NC}, ${RED}$FAIL_COUNT failed${NC}"
echo "======================================"
echo "Test details are stored in: $TEMP_DIR"
echo "======================================"

# Return non-zero exit code if any test failed
if [ $FAIL_COUNT -gt 0 ]; then
    exit 1
else
    exit 0
fi

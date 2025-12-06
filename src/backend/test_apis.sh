#!/bin/bash

# Comprehensive API testing script
BASE_AUTH="http://localhost:3001"
BASE_USER="http://localhost:3002"
BASE_STEG="http://localhost:3003"

echo "=== Starting Comprehensive API Tests ==="
echo

# Test 1: Register User
echo "TEST 1: Register new user"
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_AUTH/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "user_name": "testuser",
    "first_name": "Test",
    "last_name": "User",
    "email": "test@example.com",
    "password": "SecurePass123!",
    "is_male": true
  }')
echo "Response: $REGISTER_RESPONSE"
echo

# Extract tokens
ACCESS_TOKEN=$(echo $REGISTER_RESPONSE | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
REFRESH_TOKEN=$(echo $REGISTER_RESPONSE | grep -o '"refresh_token":"[^"]*"' | cut -d'"' -f4)
USER_ID=$(echo $REGISTER_RESPONSE | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

echo "Extracted ACCESS_TOKEN: ${ACCESS_TOKEN:0:50}..."
echo "Extracted USER_ID: $USER_ID"
echo

# Test 2: Login with email
echo "TEST 2: Login with email"
LOGIN_EMAIL_RESPONSE=$(curl -s -X POST "$BASE_AUTH/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePass123!"
  }')
echo "Response: $LOGIN_EMAIL_RESPONSE"
echo

# Test 3: Login with username
echo "TEST 3: Login with username"
LOGIN_USERNAME_RESPONSE=$(curl -s -X POST "$BASE_AUTH/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "user_name": "testuser",
    "password": "SecurePass123!"
  }')
echo "Response: $LOGIN_USERNAME_RESPONSE"
echo

# Test 4: Get current profile
echo "TEST 4: Get current user profile"
PROFILE_RESPONSE=$(curl -s -X GET "$BASE_USER/users/me" \
  -H "Authorization: Bearer $ACCESS_TOKEN")
echo "Response: $PROFILE_RESPONSE"
echo

# Test 5: Update profile
echo "TEST 5: Update user profile"
UPDATE_RESPONSE=$(curl -s -X PATCH "$BASE_USER/users/me" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "Updated",
    "last_name": "Name"
  }')
echo "Response: $UPDATE_RESPONSE"
echo

# Test 6: Refresh token
echo "TEST 6: Refresh access token"
REFRESH_RESPONSE=$(curl -s -X POST "$BASE_AUTH/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$REFRESH_TOKEN\"
  }")
echo "Response: $REFRESH_RESPONSE"
NEW_ACCESS_TOKEN=$(echo $REFRESH_RESPONSE | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
echo "New ACCESS_TOKEN: ${NEW_ACCESS_TOKEN:0:50}..."
echo

# Test 7: Invalid credentials
echo "TEST 7: Login with invalid password"
INVALID_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_AUTH/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "WrongPassword"
  }')
echo "Response (should be 401): $INVALID_LOGIN_RESPONSE"
echo

# Test 8: Access protected endpoint without token
echo "TEST 8: Access protected endpoint without token"
NO_TOKEN_RESPONSE=$(curl -s -X GET "$BASE_USER/users/me")
echo "Response (should be 401): $NO_TOKEN_RESPONSE"
echo

# Test 9: Deactivate own account
echo "TEST 9: Deactivate own account"
DEACTIVATE_RESPONSE=$(curl -s -X POST "$BASE_AUTH/auth/deactivate" \
  -H "Authorization: Bearer $ACCESS_TOKEN")
echo "Response: $DEACTIVATE_RESPONSE"
echo

# Test 10: Try to login after deactivation
echo "TEST 10: Try to login after deactivation"
DEACTIVATED_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_AUTH/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePass123!"
  }')
echo "Response (should fail): $DEACTIVATED_LOGIN_RESPONSE"
echo

echo "=== Testing Complete ==="

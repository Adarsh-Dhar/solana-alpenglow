#!/bin/bash
# Alpenglow Verification Performance Benchmark Script
# This script measures verification performance and scalability

set -e

echo "=== Alpenglow Verification Performance Benchmark ==="
echo "Timestamp: $(date)"
echo "Measuring verification performance across different configurations"
echo ""

# Create results directory
mkdir -p results/benchmark_verification
cd results/benchmark_verification

# Initialize benchmark results
echo "Validators,Slots,User_Time,Max_Memory,States_Explored,Transitions,Properties_Checked" > benchmark_results.csv

# Test different validator counts
echo "Benchmarking with different validator counts..."

for validators in 2 3 4 5 6; do
    for slots in 1 2 3; do
        echo "Testing $validators validators, $slots slots..."
        
        # Run benchmark with time and memory measurement
        /usr/bin/time -v cargo run --release --bin votor_benchmark -- --validators $validators --slots $slots --seed 12345 > benchmark_${validators}v_${slots}s.log 2>&1
        
        # Extract timing information
        user_time=$(grep "User time" benchmark_${validators}v_${slots}s.log | awk '{print $4}' | sed 's/elapsed//')
        max_memory=$(grep "Maximum resident set size" benchmark_${validators}v_${slots}s.log | awk '{print $6}')
        
        # Extract verification statistics
        states_explored=$(grep "States explored" benchmark_${validators}v_${slots}s.log | awk '{print $3}' || echo "0")
        transitions=$(grep "Transitions" benchmark_${validators}v_${slots}s.log | awk '{print $2}' || echo "0")
        properties_checked=$(grep "Properties checked" benchmark_${validators}v_${slots}s.log | awk '{print $3}' || echo "0")
        
        echo "$validators,$slots,$user_time,$max_memory,$states_explored,$transitions,$properties_checked" >> benchmark_results.csv
        
        echo "  User time: $user_time"
        echo "  Max memory: $max_memory KB"
        echo "  States explored: $states_explored"
        echo "  Transitions: $transitions"
        echo ""
    done
done

# Test scalability with larger configurations
echo "Testing scalability with larger configurations..."

# Test with more validators but fewer slots (to keep state space manageable)
for validators in 7 8 9 10; do
    slots=2
    echo "Testing $validators validators, $slots slots (scalability test)..."
    
    timeout 300 /usr/bin/time -v cargo run --release --bin votor_benchmark -- --validators $validators --slots $slots --seed 12345 > scalability_${validators}v_${slots}s.log 2>&1 || echo "Timeout after 5 minutes"
    
    if [ -f scalability_${validators}v_${slots}s.log ]; then
        user_time=$(grep "User time" scalability_${validators}v_${slots}s.log | awk '{print $4}' | sed 's/elapsed//' || echo "TIMEOUT")
        max_memory=$(grep "Maximum resident set size" scalability_${validators}v_${slots}s.log | awk '{print $6}' || echo "0")
        states_explored=$(grep "States explored" scalability_${validators}v_${slots}s.log | awk '{print $3}' || echo "0")
        transitions=$(grep "Transitions" scalability_${validators}v_${slots}s.log | awk '{print $2}' || echo "0")
        properties_checked=$(grep "Properties checked" scalability_${validators}v_${slots}s.log | awk '{print $3}' || echo "0")
        
        echo "$validators,$slots,$user_time,$max_memory,$states_explored,$transitions,$properties_checked" >> benchmark_results.csv
    fi
done

# Test simulation performance
echo "Testing simulation performance..."

# Run simulation benchmarks
for validators in 10 20 50 100; do
    echo "Simulation benchmark with $validators validators..."
    
    /usr/bin/time -v cargo run --release --bin simulation_benchmark -- --validators $validators --simulations 1000 --seed 12345 > sim_benchmark_${validators}v.log 2>&1
    
    user_time=$(grep "User time" sim_benchmark_${validators}v.log | awk '{print $4}' | sed 's/elapsed//')
    max_memory=$(grep "Maximum resident set size" sim_benchmark_${validators}v.log | awk '{print $6}')
    simulations_completed=$(grep "Simulations completed" sim_benchmark_${validators}v.log | awk '{print $3}' || echo "1000")
    
    echo "simulation,$validators,0,$user_time,$max_memory,0,0,$simulations_completed" >> benchmark_results.csv
done

# Generate performance analysis
echo ""
echo "=== Performance Analysis ==="

# Analyze state space growth
echo "State Space Growth Analysis:"
echo "Validators,Slots,States,Transitions,Time_per_State" > state_space_analysis.csv

while IFS=',' read -r validators slots user_time max_memory states_explored transitions properties_checked; do
    if [ "$validators" != "Validators" ] && [ "$states_explored" != "0" ] && [ "$user_time" != "TIMEOUT" ]; then
        # Calculate time per state (rough approximation)
        time_per_state=$(echo "scale=6; $user_time / $states_explored" | bc -l 2>/dev/null || echo "0")
        echo "$validators,$slots,$states_explored,$transitions,$time_per_state" >> state_space_analysis.csv
    fi
done < benchmark_results.csv

# Analyze memory usage
echo "Memory Usage Analysis:"
echo "Validators,Slots,Memory_KB,Memory_per_State" > memory_analysis.csv

while IFS=',' read -r validators slots user_time max_memory states_explored transitions properties_checked; do
    if [ "$validators" != "Validators" ] && [ "$states_explored" != "0" ] && [ "$max_memory" != "0" ]; then
        memory_per_state=$(echo "scale=2; $max_memory / $states_explored" | bc -l 2>/dev/null || echo "0")
        echo "$validators,$slots,$max_memory,$memory_per_state" >> memory_analysis.csv
    fi
done < benchmark_results.csv

# Generate summary report
echo ""
echo "=== Benchmark Summary ==="
echo "Total configurations tested: $(wc -l < benchmark_results.csv)"
echo "Successful verifications: $(grep -v "TIMEOUT" benchmark_results.csv | wc -l)"
echo "Timeouts: $(grep "TIMEOUT" benchmark_results.csv | wc -l)"

# Find the largest configuration that completed successfully
largest_config=$(grep -v "TIMEOUT" benchmark_results.csv | tail -n +2 | awk -F',' '{print $1","$2}' | sort -t',' -k1,1n -k2,2n | tail -1)
echo "Largest successful configuration: $largest_config"

# Find the fastest verification
fastest_time=$(grep -v "TIMEOUT" benchmark_results.csv | tail -n +2 | awk -F',' '{print $3}' | sort -n | head -1)
echo "Fastest verification time: $fastest_time"

# Find the most memory-efficient verification
min_memory=$(grep -v "TIMEOUT" benchmark_results.csv | tail -n +2 | awk -F',' '{print $4}' | sort -n | head -1)
echo "Most memory-efficient verification: $min_memory KB"

echo ""
echo "Benchmark completed. Check benchmark_results.csv for detailed results."
echo "State space analysis: state_space_analysis.csv"
echo "Memory analysis: memory_analysis.csv"

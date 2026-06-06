#!/usr/bin/env python
import subprocess
import time
import threading
import psutil
import re
import json
import os
import matplotlib.pyplot as plt

DATA_FILE = "assets/benchmark_data.json"

def run_wrk():
    print("🚀 Firing the wrk load test on localhost... (Wait 30 seconds)")
    try:
        result = subprocess.run(
            ["wrk", "-t4", "-c200", "-d30s", "http://127.0.0.1:8000/"],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"❌ Error running wrk: {e.output}")
        return ""
    except FileNotFoundError:
        print("❌ 'wrk' command not found! Please make sure it is installed (e.g. brew install wrk).")
        return ""

def parse_wrk_output(output):
    if not output:
        return 0.0, 0.0, 0.0, 0
    
    rps_match = re.search(r"Requests/sec:\s+([\d\.]+)", output)
    latency_match = re.search(r"Latency\s+([\d\.]+)(ms|s|us|m)", output)
    transfer_match = re.search(r"Transfer/sec:\s+([\d\.]+)(MB|KB|B)", output)
    errors_match = re.search(r"Socket errors:\s+connect\s+(\d+),\s+read\s+(\d+),\s+write\s+(\d+),\s+timeout\s+(\d+)", output)
    
    rps = float(rps_match.group(1)) if rps_match else 0.0
    
    latency_ms = 0.0
    if latency_match:
        val = float(latency_match.group(1))
        unit = latency_match.group(2)
        if unit == 's': val *= 1000
        elif unit == 'us': val /= 1000
        elif unit == 'm': val *= 60000
        latency_ms = val
        
    transfer_mbps = 0.0
    if transfer_match:
        val = float(transfer_match.group(1))
        unit = transfer_match.group(2)
        if unit == 'KB': val /= 1024
        elif unit == 'B': val /= (1024 * 1024)
        transfer_mbps = val
        
    errors = 0
    if errors_match:
        errors = sum(int(x) for x in errors_match.groups())
    
    return rps, latency_ms, transfer_mbps, errors

def draw_graphs(data):
    if not os.path.exists("assets"):
        os.makedirs("assets")
        
    targets = list(data.keys())
    colors = ['#4CAF50' if t == 'oxygn' else '#F44336' for t in targets]
    
    # ---------------- Graph 1: RPS ----------------
    plt.figure(figsize=(8, 6))
    rps_values = [data[t]["rps"] for t in targets]
    bars = plt.bar(targets, rps_values, color=colors)
    plt.title("Throughput: Oxygn vs Nginx (Higher is Better)", fontsize=16, fontweight='bold')
    plt.ylabel("Requests Per Second (RPS)", fontsize=12)
    plt.grid(axis='y', linestyle='--', alpha=0.7)
    for bar in bars:
        yval = bar.get_height()
        plt.text(bar.get_x() + bar.get_width()/2, yval + (max(rps_values)*0.01), 
                 f"{int(yval):,}", ha='center', va='bottom', fontweight='bold')
    plt.savefig("assets/benchmark_rps.png", bbox_inches='tight', dpi=300)
    plt.close()
    
    # ---------------- Graph 2: Latency ----------------
    plt.figure(figsize=(8, 6))
    latency_values = [data[t]["latency"] for t in targets]
    bars = plt.bar(targets, latency_values, color=colors)
    plt.title("Latency: Oxygn vs Nginx (Lower is Better)", fontsize=16, fontweight='bold')
    plt.ylabel("Average Latency (ms)", fontsize=12)
    plt.grid(axis='y', linestyle='--', alpha=0.7)
    for bar in bars:
        yval = bar.get_height()
        plt.text(bar.get_x() + bar.get_width()/2, yval + (max(latency_values)*0.01), 
                 f"{yval:.1f} ms", ha='center', va='bottom', fontweight='bold')
    plt.savefig("assets/benchmark_latency.png", bbox_inches='tight', dpi=300)
    plt.close()
    
    # ---------------- Graph 3: Transfer Rate ----------------
    plt.figure(figsize=(8, 6))
    transfer_values = [data[t]["transfer_mbps"] for t in targets]
    bars = plt.bar(targets, transfer_values, color=colors)
    plt.title("Data Transfer Rate (Higher is Better)", fontsize=16, fontweight='bold')
    plt.ylabel("Transfer Rate (MB/s)", fontsize=12)
    plt.grid(axis='y', linestyle='--', alpha=0.7)
    for bar in bars:
        yval = bar.get_height()
        plt.text(bar.get_x() + bar.get_width()/2, yval + (max(transfer_values)*0.01), 
                 f"{yval:.2f} MB/s", ha='center', va='bottom', fontweight='bold')
    plt.savefig("assets/benchmark_transfer.png", bbox_inches='tight', dpi=300)
    plt.close()
    
    # ---------------- Graph 4: Socket Errors ----------------
    plt.figure(figsize=(8, 6))
    error_values = [data[t]["errors"] for t in targets]
    bars = plt.bar(targets, error_values, color=colors)
    plt.title("Socket Stability: Dropped Connections (Lower is Better)", fontsize=16, fontweight='bold')
    plt.ylabel("Total Socket Errors", fontsize=12)
    plt.grid(axis='y', linestyle='--', alpha=0.7)
    for bar in bars:
        yval = bar.get_height()
        plt.text(bar.get_x() + bar.get_width()/2, yval + (max(error_values)*0.01 if max(error_values) > 0 else 0.1), 
                 f"{int(yval)}", ha='center', va='bottom', fontweight='bold')
    plt.savefig("assets/benchmark_errors.png", bbox_inches='tight', dpi=300)
    plt.close()
    
    # ---------------- Graph 5: Memory Over Time ----------------
    plt.figure(figsize=(10, 6))
    for t in targets:
        mem_data = data[t]["memory_mb"]
        if len(mem_data) > 4: mem_data = mem_data[4:]
        if len(mem_data) > 60: mem_data = mem_data[:60]
        times = [i * 0.5 for i in range(len(mem_data))]
        color = '#4CAF50' if t == 'oxygn' else '#F44336'
        plt.plot(times, mem_data, label=f"{t.capitalize()} (Avg: {sum(mem_data)/len(mem_data) if mem_data else 0:.1f} MB)", color=color, linewidth=3)
    plt.title("Memory Efficiency Over Time (Lower is Better)", fontsize=16, fontweight='bold')
    plt.xlabel("Time (Seconds)", fontsize=12)
    plt.ylabel("RAM Usage (MB)", fontsize=12)
    plt.legend()
    plt.grid(True, linestyle='--', alpha=0.7)
    plt.savefig("assets/benchmark_memory.png", bbox_inches='tight', dpi=300)
    plt.close()
    
    # ---------------- Graph 6: CPU Over Time ----------------
    plt.figure(figsize=(10, 6))
    for t in targets:
        cpu_data = data[t]["cpu_percent"]
        if len(cpu_data) > 4: cpu_data = cpu_data[4:]
        if len(cpu_data) > 60: cpu_data = cpu_data[:60]
        times = [i * 0.5 for i in range(len(cpu_data))]
        color = '#4CAF50' if t == 'oxygn' else '#F44336'
        plt.plot(times, cpu_data, label=f"{t.capitalize()} (Avg: {sum(cpu_data)/len(cpu_data) if cpu_data else 0:.1f} %)", color=color, linewidth=3)
    plt.title("CPU Usage Over Time (Lower is Better)", fontsize=16, fontweight='bold')
    plt.xlabel("Time (Seconds)", fontsize=12)
    plt.ylabel("CPU Usage (%)", fontsize=12)
    plt.legend()
    plt.grid(True, linestyle='--', alpha=0.7)
    plt.savefig("assets/benchmark_cpu.png", bbox_inches='tight', dpi=300)
    plt.close()
    
    print("\n✅ All 6 Graphs successfully saved to the assets/ folder!")

def run_benchmark_for_target(target, data):
    print(f"\n{'='*60}")
    print(f"🎯 STARTING BENCHMARK FOR: {target.upper()}")
    print(f"{'='*60}")
    
    data[target] = {"rps": 0, "latency": "", "transfer_mbps": 0, "errors": 0, "memory_mb": [], "cpu_percent": []}
    
    print(f"🔄 Booting up {target} proxy...")
    if target == "oxygn":
        proc = subprocess.Popen(["cargo", "run", "--release"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    else:
        nginx_conf = os.path.abspath("utils/nginx.conf")
        proc = subprocess.Popen(["nginx", "-c", nginx_conf, "-g", "daemon off;"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
    time.sleep(3) # Wait for it to bind ports
    
    stop_recording = False
    
    def record_metrics():
        try:
            p = psutil.Process(proc.pid)
            p.cpu_percent(interval=None) # prime the cpu tracker
            while not stop_recording and proc.poll() is None:
                mem_mb = p.memory_info().rss / (1024 * 1024)
                cpu_p = p.cpu_percent(interval=None)
                data[target]["memory_mb"].append(mem_mb)
                data[target]["cpu_percent"].append(cpu_p)
                time.sleep(0.5)
        except Exception as e:
            pass

    t = threading.Thread(target=record_metrics)
    t.start()
    
    output = run_wrk()
    
    stop_recording = True
    proc.terminate()
    try:
        proc.wait(timeout=3)
    except:
        proc.kill()
        
    t.join()
    
    if target == "nginx":
        os.system("pkill nginx > /dev/null 2>&1")
        time.sleep(1)
    
    rps, latency, transfer, errors = parse_wrk_output(output)
    print(f"\n📊 Extracted Results for {target}:")
    print(f"RPS: {rps:,}")
    print(f"Latency: {latency:.1f} ms")
    print(f"Transfer: {transfer:.2f} MB/s")
    print(f"Errors: {errors}")
    
    data[target]["rps"] = rps
    data[target]["latency"] = latency
    data[target]["transfer_mbps"] = transfer
    data[target]["errors"] = errors
    
def main():
    if not os.path.exists("assets"):
        os.makedirs("assets")
        
    data = {}
    
    run_benchmark_for_target("oxygn", data)
    
    print("\n⏳ Cooling down for 3 seconds before next test...")
    time.sleep(3)
    
    run_benchmark_for_target("nginx", data)
    
    with open(DATA_FILE, "w") as f:
        json.dump(data, f, indent=4)
        
    draw_graphs(data)
    print("\n🎉 ALL TESTS COMPLETE! Check your beautiful graphs!")

if __name__ == "__main__":
    main()

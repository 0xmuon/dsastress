#include <bits/stdc++.h>
using namespace std;

int main() {
  ios::sync_with_stdio(false);
  cin.tie(nullptr);

  int n;
  long long K;
  if (!(cin >> n >> K)) return 0;
  vector<long long> a(n);
  for (int i = 0; i < n; i++) cin >> a[i];

  sort(a.begin(), a.end());
  int i = 0, j = n - 1;
  long long ans = 0;

  while (i < j) {
    long long sum = a[i] + a[j];
    if (sum == K) {
      if (a[i] == a[j]) {
        long long m = (long long)j - i + 1;
        ans += m * (m - 1) / 2;
        break;
      }
      long long lv = a[i], rv = a[j];
      long long c1 = 0, c2 = 0;
      while (i <= j && a[i] == lv) {
        c1++;
        i++;
      }
      while (j >= i && a[j] == rv) {
        c2++;
        j--;
      }
      ans += c1 * c2;
    } else if (sum < K) {
      i++;
    } else {
      j--;
    }
  }

  cout << ans << "\n";
  return 0;
}


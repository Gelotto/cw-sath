from datetime import datetime, timedelta

unbonding_duration = timedelta(hours=1)

t_deleg_1 = datetime.now()
t_withdraw_1 = t_deleg_1 + unbonding_duration
amount1 = 50

t_deleg_2 = t_deleg_1 + timedelta(minutes=45)
t_withdraw_2 = t_deleg_2 + unbonding_duration
amount2 = 5

total_amount = amount1 + amount2

dt1 = ((t_withdraw_1 - t_deleg_2).total_seconds() * amount1/total_amount)
dt2 = ((unbonding_duration).total_seconds() * amount2/total_amount)

print(dt1, dt2, (dt1+dt2)/2)
print(t_deleg_2 + timedelta(seconds=(dt1+dt2)/2))
print(t_deleg_2 + unbonding_duration)
print(datetime.fromtimestamp(
    t_withdraw_1.timestamp()*(amount1/total_amount) + t_withdraw_2.timestamp() * (amount2/total_amount)))


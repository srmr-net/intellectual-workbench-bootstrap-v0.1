"""Independent reference: high-precision discounted cash-flow summation.

This is intentionally different from the guest's exact power/rational formula.
Only fixture/oracle data are emitted; the host never uses this to compute results.
"""
from decimal import Decimal,localcontext,ROUND_HALF_UP
from pathlib import Path
import json,random
ROOT=Path(__file__).resolve().parents[2]

def payment(value):
    with localcontext() as c:
        c.prec=90
        principal=Decimal(value['principal'])
        monthly=Decimal(value['annual_rate_percent'])/Decimal(1200)
        discount=Decimal(1)/(Decimal(1)+monthly)
        factor=Decimal(1)
        annuity=Decimal(0)
        for _ in range(value['term_months']):
            factor*=discount
            annuity+=factor
        return str((principal/annuity).quantize(Decimal('0.01'),rounding=ROUND_HALF_UP))

def main():
    data=json.loads((ROOT/'fixtures/mortgage-vectors.json').read_text())
    cases=[]
    for item in data['positive']:
        result=payment(item['input'])
        assert result==item['expected']['monthly_payment'],item['id']
        cases.append(item)
    boundaries=[('round-to-zero','0.01','0',600),('maximum-payment','1000000000','100',1),
                ('maximum-tiny-rate','1000000000','0.000001',600),('max-rate-long','1000000000','100',600),
                ('max-coprime-600','1000000000','99.999997',600),
                ('max-coprime-599','1000000000','99.999997',599),
                ('max-coprime-511','1000000000','99.999997',511),
                ('tie-below','0.99','0',8),('tie-above','1.01','0',8)]
    for name,p,r,n in boundaries:
        value={'principal':p,'annual_rate_percent':r,'term_months':n}
        cases.append({'id':name,'input':value,'expected':{'monthly_payment':payment(value)}})
    randomizer=random.Random(20261026)
    for n in range(16):
        cents=randomizer.randint(1,100_000_000_000)
        rate=randomizer.randint(0,100_000_000)
        value={'principal':f'{cents//100}.{cents%100:02d}',
               'annual_rate_percent':f'{rate//1_000_000}.{rate%1_000_000:06d}',
               'term_months':randomizer.randint(1,600)}
        cases.append({'id':f'seeded-{n}','input':value,'expected':{'monthly_payment':payment(value)}})
    target=ROOT/'target/mortgage-reference.json';target.parent.mkdir(exist_ok=True)
    target.write_text(json.dumps({'precision':90,'rounding':'ROUND_HALF_UP','algorithm':'discounted cashflow sum',
                                 'seed':20261026,'cases':cases},indent=2)+'\n')
    print(f'PASS: eight original vectors, nine boundaries and sixteen independent reference cases: {target.relative_to(ROOT)}')
if __name__=='__main__':main()

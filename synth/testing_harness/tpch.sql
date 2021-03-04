
-- Drop table

-- DROP TABLE public.lineitem;

CREATE TABLE public.lineitem (
	l_orderkey int4 NOT NULL,
	l_partkey int4 NOT NULL,
	l_suppkey int4 NOT NULL,
	l_linenumber int4 NOT NULL,
	l_quantity numeric(15,2) NOT NULL,
	l_extendedprice numeric(15,2) NOT NULL,
	l_discount numeric(15,2) NOT NULL,
	l_tax numeric(15,2) NOT NULL,
	l_returnflag bpchar(1) NOT NULL,
	l_linestatus bpchar(1) NOT NULL,
	l_shipdate date NOT NULL,
	l_commitdate date NOT NULL,
	l_receiptdate date NOT NULL,
	l_shipinstruct bpchar(25) NOT NULL,
	l_shipmode bpchar(10) NOT NULL,
	l_comment varchar(44) NOT NULL
);

-- Drop table

-- DROP TABLE public.nation;

CREATE TABLE public.nation (
	n_nationkey int4 NOT NULL,
	n_name bpchar(25) NOT NULL,
	n_regionkey int4 NOT NULL,
	n_comment varchar(152) NULL,
	CONSTRAINT nation_pkey PRIMARY KEY (n_nationkey)
);

-- Drop table

-- DROP TABLE public.orders;

CREATE TABLE public.orders (
	o_orderkey int4 NOT NULL,
	o_custkey int4 NOT NULL,
	o_orderstatus bpchar(1) NOT NULL,
	o_totalprice numeric(15,2) NOT NULL,
	o_orderdate date NOT NULL,
	o_orderpriority bpchar(15) NOT NULL,
	o_clerk bpchar(15) NOT NULL,
	o_shippriority int4 NOT NULL,
	o_comment varchar(79) NOT NULL
);

-- Drop table

-- DROP TABLE public.part;

CREATE TABLE public.part (
	p_partkey int4 NOT NULL,
	p_name varchar(55) NOT NULL,
	p_mfgr bpchar(25) NOT NULL,
	p_brand bpchar(10) NOT NULL,
	p_type varchar(25) NOT NULL,
	p_size int4 NOT NULL,
	p_container bpchar(10) NOT NULL,
	p_retailprice numeric(15,2) NOT NULL,
	p_comment varchar(23) NOT NULL
);

-- Drop table

-- DROP TABLE public.partsupp;

CREATE TABLE public.partsupp (
	ps_partkey int4 NOT NULL,
	ps_suppkey int4 NOT NULL,
	ps_availqty int4 NOT NULL,
	ps_supplycost numeric(15,2) NOT NULL,
	ps_comment varchar(199) NOT NULL
);

-- Drop table

-- DROP TABLE public.region;

CREATE TABLE public.region (
	r_regionkey int4 NOT NULL,
	r_name bpchar(25) NOT NULL,
	r_comment varchar(152) NULL
);

-- Drop table

-- DROP TABLE public.supplier;

CREATE TABLE public.supplier (
	s_suppkey int4 NOT NULL,
	s_name bpchar(25) NOT NULL,
	s_address varchar(40) NOT NULL,
	s_nationkey int4 NOT NULL,
	s_phone bpchar(15) NOT NULL,
	s_acctbal numeric(15,2) NOT NULL,
	s_comment varchar(101) NOT NULL
);

-- Drop table

-- DROP TABLE public.customer;

CREATE TABLE public.customer (
	c_custkey int4 NOT NULL,
	c_name varchar(25) NOT NULL,
	c_address varchar(40) NOT NULL,
	c_nationkey int4 NOT NULL,
	c_phone bpchar(15) NOT NULL,
	c_acctbal numeric(15,2) NOT NULL,
	c_mktsegment bpchar(10) NOT NULL,
	c_comment varchar(117) NOT NULL,
	CONSTRAINT customer_pkey PRIMARY KEY (c_custkey)
);

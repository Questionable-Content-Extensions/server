using System;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using AutoMapper;
using MediatR;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Logs.Models;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Logs.Queries.GetLogs
{
	public class GetLogsQueryHandler : IRequestHandler<GetLogsQuery, LogEntriesDto>
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;

		public GetLogsQueryHandler(
			DomainDbContext context,
			IMapper mapper
		)
		{
			_context = context;
			_mapper = mapper;
		}

		public async Task<LogEntriesDto> Handle(GetLogsQuery request, CancellationToken cancellationToken)
		{
			var logEntries = await _context.LogEntries
				.OrderByDescending(l => l.DateTime)
				.Skip((request.Page - 1) * request.PageSize)
				.Take(request.PageSize)
				.Select(l => new LogDto { Identifier = l.Token.Identifier, DateTime = l.DateTime, Action = l.Action }).ToArrayAsync();
			var logEntryCount = await _context.LogEntries.CountAsync();
			return new LogEntriesDto
			{
				LogEntries = logEntries,
				Page = request.Page,
				LogEntryCount = logEntryCount,
				PageCount = (int)Math.Ceiling(logEntryCount * 1.0 / request.PageSize)
			};
		}
	}
}
